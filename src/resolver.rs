use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use dnrs::{
    DnsError, Flags, Header, Message, Name, Networkable, Question, RecordData, RecordType,
    ResourceRecord,
};
use itertools::{Either, Itertools};
use rand::seq::SliceRandom;
use tokio::net::UdpSocket;
use tracing::{debug, info, instrument, trace, warn};

use crate::util::set_response_flags;

mod cache;
use cache::Cache;

// TODO: Make this a list of hosts?
const ROOT_NAMESERVERS: [IpAddr; 13] = [
    IpAddr::V4(Ipv4Addr::new(198, 41, 0, 4)),
    IpAddr::V4(Ipv4Addr::new(199, 9, 14, 201)),
    IpAddr::V4(Ipv4Addr::new(192, 33, 4, 12)),
    IpAddr::V4(Ipv4Addr::new(199, 7, 91, 13)),
    IpAddr::V4(Ipv4Addr::new(192, 203, 230, 10)),
    IpAddr::V4(Ipv4Addr::new(192, 5, 5, 241)),
    IpAddr::V4(Ipv4Addr::new(192, 112, 36, 4)),
    IpAddr::V4(Ipv4Addr::new(198, 97, 190, 53)),
    IpAddr::V4(Ipv4Addr::new(192, 36, 148, 17)),
    IpAddr::V4(Ipv4Addr::new(192, 58, 128, 30)),
    IpAddr::V4(Ipv4Addr::new(193, 0, 14, 129)),
    IpAddr::V4(Ipv4Addr::new(199, 7, 83, 42)),
    IpAddr::V4(Ipv4Addr::new(202, 12, 27, 33)),
];

pub async fn run(ip: &str, port: u16) {
    info!("Starting udp server");

    // TODO: Spawn 2 tasks for tcp and udp
    let sock = UdpSocket::bind((ip, port))
        .await
        .expect("Couldn't run server");

    let cache = Arc::new(Mutex::new(Cache::new()));

    let sock = Arc::new(sock);

    loop {
        // http://www.dnsflagday.net/2020/
        let mut buf = [0; 1232];

        // TODO: Handle if this errors
        let (len, addr) = sock.recv_from(&mut buf).await.unwrap();
        info!(address=?addr, "received request");

        let sock = Arc::clone(&sock);
        let cache = Arc::clone(&cache);

        tokio::spawn(async move {
            if let Some(response) = handle_request(&buf[0..len], cache).await {
                sock.send_to(&response.to_bytes(), addr).await.ok();
            }
        });
    }
}

#[instrument(skip_all)]
async fn handle_request(data: &[u8], cache: Arc<Mutex<Cache>>) -> Option<Message> {
    let mut request = Message::from_bytes(&mut Cursor::new(data)).unwrap();
    debug!(?request, "parsed request");

    if request.header.flags.qr() {
        warn!(?request, "discarding request");
        return None;
    }

    if request.header.num_questions != 1 || request.header.flags.opcode() != 0 {
        warn!(?request, "unimplemented request");
        let mut flags = set_response_flags(request.header.flags);
        flags.set_rcode(4);

        let header = Header::new(request.header.id, flags);
        return Some(Message::new(header));
    }

    let question = request.questions.remove(0);

    // TODO: If rd is false check cache, otherwise resolve

    let result = resolve(question.clone(), Arc::clone(&cache)).await;

    if let Ok(records) = result {
        let flags = set_response_flags(request.header.flags);
        let header = Header::new(request.header.id, flags);

        let mut response = Message::new(header);
        response.add_question(question);
        for record in records {
            response.add_answer(record);
        }

        Some(response)
    } else {
        warn!("responding with error");
        let mut flags = set_response_flags(request.header.flags);
        flags.set_rcode(2);

        let header = Header::new(request.header.id, flags);

        let mut response = Message::new(header);
        response.add_question(question);

        Some(response)
    }
}

#[instrument(skip(cache), ret, err(Debug))]
#[async_recursion]
pub async fn resolve(
    question: Question,
    cache: Arc<Mutex<Cache>>,
) -> Result<Vec<ResourceRecord>, DnsError> {
    let flags = Flags::default();
    let id = rand::random::<u16>();
    let header = Header::new(id, flags);
    let mut query = Message::new(header);
    query.add_question(question.clone());

    let mut response = Vec::new();

    let mut buf = [0; 1024];

    let mut nameserver = (
        Name::new(""),
        *ROOT_NAMESERVERS.choose(&mut rand::thread_rng()).unwrap(),
    );

    let sock = UdpSocket::bind(("0.0.0.0", 0)).await.unwrap();

    // TODO: Some kind of work-limiting mechanism
    loop {
        let (ns_name, ns_ip) = nameserver;

        debug!(?ns_name, ?ns_ip, "querying nameserver");
        sock.send_to(&query.to_bytes(), (ns_ip, 53)).await.unwrap();

        // TODO: Need to do validation of this message
        let response_len = sock.recv(&mut buf).await?;
        let mut cursor = Cursor::new(&buf[..response_len]);
        let mut message = Message::from_bytes(&mut cursor).unwrap();

        debug!("received response from nameserver");
        trace!(?message);

        if message.header.num_answers != 0 {
            debug!(?message.answers, "received answers from nameserver");
            if let Some(idx) = message
                .answers
                .iter()
                .position(|rr| rr.type_ == question.type_)
            {
                response.push(message.answers.remove(idx));
                return Ok(response);
            }

            // If we don't have an answer that matches the question
            if let Some(rr) = message
                .answers
                .iter()
                .find(|rr| rr.type_ == RecordType::Cname)
            {
                response.push(rr.clone());

                let RecordData::Cname(name) = &rr.data else {
                    unreachable!();
                };

                info!("received cname from nameserver, re-starting resolution process");
                let answer = resolve(
                    Question::new(name.clone(), question.type_),
                    Arc::clone(&cache),
                )
                .await?;

                response.extend(answer);

                return Ok(response);
            }

            panic!();
        }

        let (mut resolved, mut unresolved): (Vec<_>, Vec<_>) = {
            // Lock once so we don't lock and unlock every iteration
            let cache = cache.lock().unwrap();

            message
                .authorities
                .iter()
                .filter_map(|rr| {
                    if let RecordData::Ns(name) = &rr.data {
                        Some(name)
                    } else {
                        None
                    }
                })
                .partition_map(|name| {
                    // If the ip is in the additionals
                    if let Some(ip) = find_ip(name, &message.additionals) {
                        return Either::Left((name.clone(), ip));
                    }

                    // If the ip is in the cache
                    if let Some(cached_rrs) = cache.get_record_set(name) {
                        if let Some(ip) = find_ip(name, &cached_rrs.iter().cloned().collect_vec()) {
                            return Either::Left((name.clone(), ip));
                        }
                    }

                    Either::Right(name.clone())
                })
        };

        if let Some(host) = resolved.pop() {
            nameserver = host;
            continue;
        } else if let Some(name) = unresolved.pop() {
            let answer = resolve(
                Question::new(name.clone(), RecordType::A),
                Arc::clone(&cache),
            )
            .await?;

            let ip = find_ip(&name, &answer).ok_or(DnsError::ServerFailure(
                "failed to resolve next nameserver".to_owned(),
            ))?;

            nameserver = (name, ip);
            continue;
        }

        let soa = message
            .authorities
            .iter()
            .find(|rr| rr.type_ == RecordType::Soa);

        if let Some(soa_record) = soa {
            response.push(soa_record.clone());
            return Ok(response);
        }

        panic!()
    }
}

fn find_iter<'a, I>(
    name: Name,
    records: I,
    t: Option<RecordType>,
) -> impl Iterator<Item = &'a ResourceRecord>
where
    I: IntoIterator<Item = &'a ResourceRecord>,
{
    records.into_iter().filter(move |rr| {
        if let Some(type_filter) = t {
            rr.name == name && rr.type_ == type_filter
        } else {
            rr.name == name
        }
    })
}

fn find_ip(name: &Name, rr_set: &[ResourceRecord]) -> Option<IpAddr> {
    for rr in rr_set {
        if &rr.name != name {
            continue;
        }

        match rr.data {
            RecordData::A(ip) => return Some(IpAddr::V4(ip)),
            // RecordData::Aaaa(ip) => return Some(IpAddr::V6(ip)),
            _ => {}
        }
    }

    None
}
