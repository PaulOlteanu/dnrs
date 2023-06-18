use std::io::Cursor;
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use dnrs::{
    DnsError, Flags, Header, Message, Name, Networkable, Question, RecordData, RecordType,
    ResourceRecord,
};
use tokio::net::UdpSocket;
use tracing::{debug, debug_span, info, instrument, span, trace, warn, Instrument, Level};

use crate::util::set_response_flags;

mod ns_queue;
use ns_queue::NsQueue;

mod cache;
use cache::Cache;

mod host;
use host::Host;

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

    let mut ns_queue = NsQueue::seeded();

    {
        let cache = cache.lock().unwrap();

        for (level, subdomain) in question.name.iter_subdomains().enumerate() {
            let level = level + 1;

            let Some(authorities) = cache.get(&Name::new(&subdomain)) else {
                continue;
            };

            let authorities: Vec<Host> = authorities
                .iter()
                .filter_map(|r| match &r.data {
                    RecordData::Ns(name) => {
                        if let Some(records) = cache.get(name) {
                            // TODO: If we don't have an A/AAAA record for this, add the name so we can resolve it and then use it
                            for record in records {
                                match record.data {
                                    RecordData::A(addr) => return Some(addr.into()),
                                    // RecordData::Aaaa(addr) => return Some(addr.into()),
                                    _ => {}
                                }
                            }
                        }

                        None
                    }
                    _ => None,
                })
                .collect();

            ns_queue.insert_multiple(&authorities, level);
        }
    }

    let sock = UdpSocket::bind(("0.0.0.0", 0)).await.unwrap();

    // TODO: Some kind of work-limiting mechanism
    loop {
        let closest_nameserver = ns_queue.pop();
        let ip = if let Some(ip) = closest_nameserver.get_ip() {
            ip
        } else {
            let resolution = resolve(
                Question::new(closest_nameserver.name().clone().unwrap(), RecordType::A),
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

        for authority in message.authorities {
            if let RecordData::Ns(ref name) = authority.data {
                if let Some(addr) = message
                    .additionals
                    .iter()
                    .find(|r| &r.name == name && r.type_ == RecordType::A)
                {
                    if let RecordData::A(ip) = addr.data {
                        let level = authority.name.matching_level(&question.name);
                        let host: Host = (addr.name.clone(), Some(ip), None).into();
                        debug!(?host, level, "adding host to ns queue");
                        ns_queue.insert(host, level);
                    } else {
                        unreachable!()
                    }
                } else {
                    // TODO: Check the cache
                    // The queue should also  have separate stores at each level
                    // for servers with only names, and servers with names & ips
                    // so that we pick from the resolved ones first
                    let level = authority.name.matching_level(name);
                    ns_queue.insert(name.clone(), level);
                    continue;
                }
            }
        }
    }
}
