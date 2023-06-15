use std::io::Cursor;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use dnrs::cache::Cache;
use dnrs::{
    DnsError, Flags, Header, Message, Name, Networkable, Question, RecordData, RecordType,
    ResourceRecord,
};
use tokio::net::UdpSocket;

use crate::util::set_response_flags;

mod ns_queue;
use ns_queue::NsQueue;

pub async fn run(ip: &str, port: u16) {
    let sock = UdpSocket::bind((ip, port))
        .await
        .expect("Couldn't run server");

    let cache = Arc::new(Mutex::new(Cache::new()));

    let sock = Arc::new(sock);

    // TODO: Spawn 2 tasks for tcp and udp
    loop {
        let sock = Arc::clone(&sock);
        let cache = Arc::clone(&cache);

        // http://www.dnsflagday.net/2020/
        let mut buf = [0; 1232];

        // TODO: Handle if this errors
        let (len, addr) = sock.recv_from(&mut buf).await.unwrap();

        tokio::spawn(async move {
            if let Some(response) = handle_request(cache, &buf[0..len]).await {
                sock.send_to(&response.to_bytes(), addr).await.ok();
            }
        });
    }
}

async fn handle_request(cache: Arc<Mutex<Cache>>, data: &[u8]) -> Option<Message> {
    let mut request = Message::from_bytes(&mut Cursor::new(data)).unwrap();

    if request.header.flags.qr() || request.header.flags.z() != 0 {
        println!("Discarding request");
        return None;
    }

    if request.header.num_questions != 1 || request.header.flags.opcode() != 0 {
        println!("Received unimplemented request");
        let mut flags = set_response_flags(request.header.flags);
        flags.set_rcode(4);

        let header = Header::new(request.header.id, flags);
        return Some(Message::new(header));
    }

    let question = request.questions.remove(0);

    // TODO: If rd is false check cache, otherwise resolve

    let result = resolve(question.clone(), Arc::clone(&cache)).await;

    if let Ok(record) = result {
        // {
        //     let mut cache = cache.lock().unwrap();
        //     cache.insert(CacheKey::from(&record), record.clone());
        // }

        let flags = set_response_flags(request.header.flags);
        let header = Header::new(request.header.id, flags);

        let mut response = Message::new(header);
        response.add_question(question);
        response.add_answer(record);

        Some(response)
    } else {
        let mut flags = set_response_flags(request.header.flags);
        flags.set_rcode(2);

        let header = Header::new(request.header.id, flags);

        let mut response = Message::new(header);
        response.add_question(question);

        Some(response)
    }
}

#[async_recursion]
pub async fn resolve(
    question: Question,
    cache: Arc<Mutex<Cache>>,
) -> Result<ResourceRecord, DnsError> {
    let flags = Flags::default();
    let id = rand::random::<u16>();
    let header = Header::new(id, flags);
    let mut query = Message::new(header);
    query.add_question(question.clone());

    let mut buf = [0; 1024];

    let mut ns_queue = NsQueue::seeded();

    {
        let cache = cache.lock().unwrap();

        let mut level = 0;

        for subdomain in question.name.iter_subdomains() {
            level += 1;

            let Some(authorities) = cache.get(&Name::new(&subdomain)) else {
                continue;
            };

            let authorities: Vec<IpAddr> = authorities
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

        sock.send_to(&query.to_bytes(), (closest_nameserver, 53))
            .await
            .unwrap();

        // TODO: Need to do validation of this message
        let response_len = sock.recv(&mut buf).await?;
        let mut cursor = Cursor::new(&buf[..response_len]);
        let mut message = Message::from_bytes(&mut cursor).unwrap();

        if message.header.num_answers == 1 {
            return Ok(message.answers.remove(0));
        }

        for authority in message.authorities {
            if let RecordData::Ns(ref name) = authority.data {
                let addr = if let Some(addr) = message
                    .additionals
                    .iter()
                    .find(|r| &r.name == name && r.type_ == RecordType::A)
                {
                    if let RecordData::A(addr) = addr.data {
                        Some(addr.into())
                    } else {
                        unreachable!()
                    }
                } else {
                    // TODO: Check the cache or query
                    let question = Question::new(&name.name, RecordType::A)?;
                    let temp = resolve(question.clone(), Arc::clone(&cache)).await?;

                    if let RecordData::A(data) = temp.data {
                        Some(data.into())
                    } else {
                        None
                    }
                };

                if let Some(addr) = addr {
                    let level = authority.name.matching_level(&question.name);
                    ns_queue.insert(addr, level);
                }
            }
        }
    }
}
