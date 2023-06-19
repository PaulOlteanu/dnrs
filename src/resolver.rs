use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use dnrs::{
    DnsError, Flags, Header, Message, Name, Networkable, Question, RecordData, RecordType,
    ResourceRecord,
};
use tokio::net::UdpSocket;
use tracing::{debug, info, instrument, trace, warn};

use crate::util::set_response_flags;

mod ns_queue;
use ns_queue::NsQueue;

mod cache;
use cache::Cache;

// TODO: Make this a list of hosts
const ROOT_NAMESERVERS: [Ipv4Addr; 13] = [
    Ipv4Addr::new(198, 41, 0, 4),
    Ipv4Addr::new(199, 9, 14, 201),
    Ipv4Addr::new(192, 33, 4, 12),
    Ipv4Addr::new(199, 7, 91, 13),
    Ipv4Addr::new(192, 203, 230, 10),
    Ipv4Addr::new(192, 5, 5, 241),
    Ipv4Addr::new(192, 112, 36, 4),
    Ipv4Addr::new(198, 97, 190, 53),
    Ipv4Addr::new(192, 36, 148, 17),
    Ipv4Addr::new(192, 58, 128, 30),
    Ipv4Addr::new(193, 0, 14, 129),
    Ipv4Addr::new(199, 7, 83, 42),
    Ipv4Addr::new(202, 12, 27, 33),
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

    let mut ns_queue = NsQueue::new();
    // TODO: Make this better
    for ns in ROOT_NAMESERVERS {
        ns_queue.insert(Name::new(""), Some(IpAddr::V4(ns)), 0);
    }

    {
        let cache = cache.lock().unwrap();

        for (level, subdomain) in question.name.iter_subdomains().enumerate() {
            let level = level + 1;

            let Some(authorities) = cache.get(&Name::new(&subdomain)) else {
                continue;
            };

            for authority in authorities.iter().filter_map(|r| match &r.data {
                RecordData::Ns(name) => {
                    if let Some(records) = cache.get(name) {
                        // TODO: If we don't have an A/AAAA record for this, add the name so we can resolve it and then use it
                        for record in records {
                            if let RecordData::A(ip) = record.data {
                                return Some((name.clone(), Some(IpAddr::V4(ip))));
                            }
                        }
                    }

                    None
                }

                _ => None,
            }) {
                ns_queue.insert(authority.0, authority.1, level);
            }
        }
    }

    let sock = UdpSocket::bind(("0.0.0.0", 0)).await.unwrap();

    // TODO: Some kind of work-limiting mechanism
    loop {
        let Some(closest_nameserver) = ns_queue.pop() else {
            return Err(DnsError::ServerFailure("failed to resolve".to_owned()))
        };

        let ip = if let Some(ip) = closest_nameserver.1 {
            ip
        } else {
            let resolution = resolve(
                Question::new(closest_nameserver.0.clone(), RecordType::A),
                Arc::clone(&cache),
            )
            .await?;

            let Some(ResourceRecord{data: RecordData::A(ip), ..}) = resolution
            .iter()
            .find(|rr| rr.type_ == RecordType::A) else {
                continue;
            };
            (*ip).into()
        };

        debug!(address=?closest_nameserver, "querying nameserver");
        sock.send_to(&query.to_bytes(), (ip, 53)).await.unwrap();

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
                let answer = resolve(Question::new(name.clone(), question.type_), cache).await?;

                response.extend(answer);

                return Ok(response);
            }

            panic!();
        }

        let mut ns_records = message
            .authorities
            .iter()
            .filter(|rr| rr.type_ == RecordType::Ns)
            .peekable();

        if ns_records.peek().is_some() {
            for record in ns_records {
                let RecordData::Ns(authority_name) = &record.data else {
                    unreachable!()
                };

                if let Some(ResourceRecord {
                    data: RecordData::A(ip),
                    ..
                }) = message
                    .additionals
                    .iter()
                    .find(|r| &r.name == authority_name && r.type_ == RecordType::A)
                {
                    let level = record.name.matching_level(&question.name);
                    trace!(?authority_name, level, "adding host to ns queue");
                    ns_queue.insert(authority_name.clone(), Some(IpAddr::V4(*ip)), level);
                } else {
                    // TODO: Check the cache
                    // The queue should also have separate stores at each level
                    // for servers with only names, and servers with names & ips
                    // so that we pick from the resolved ones first
                    let level = record.name.matching_level(&question.name);
                    ns_queue.insert(authority_name.clone(), None, level);
                    continue;
                }
            }

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
