use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use dnrs::cache::{Cache, CacheKey};
use dnrs::{
    DnsError, Flags, Header, Message, Name, Networkable, Question, RecordData, RecordType,
    ResourceRecord,
};
use tokio::net::UdpSocket;

use crate::util::set_response_flags;

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
    IpAddr::V4(Ipv4Addr::new(202, 12, 24, 33)),
];

struct NsQueue {
    queue: Vec<Vec<IpAddr>>,
}

impl NsQueue {
    pub fn new() -> Self {
        Self {
            queue: vec![ROOT_NAMESERVERS.into()],
        }
    }
}

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

    // Validate request
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
    // let flags = Flags::default();
    // let id = rand::random::<u16>();
    // let header = Header::new(id, flags);
    // let mut query = Message::new(header);
    // query.questions.push(question);

    // let mut buf = [0; 1024];

    // This starts with the root nameservers prepopulated
    let mut ns_queue = NsQueue::new();

    {
        let cache = cache.lock().unwrap();
        for subdomain in question.name.iter_subdomains() {
            let key = CacheKey::new(1, RecordType::Ns, Name::new(&subdomain));

            if let Some(authorities) = cache.get(&key) {
                // let authorities = authorities
                //     .iter()
                //     .map(|r| {
                //         if let RecordData::A(a) = r.data {

                //     }});
                // ns_queue.queue.push(vec![])
            }
        }
    }

    loop {
        // Check the cache for the closest subdomain authority(ies)
        // Add them

        // sock.send_to(&query.to_bytes(), (nameserver, 53))
        //     .await
        //     .unwrap();

        // // TODO: Need to do validation of this message
        // let response_len = sock.recv(&mut buf).await?;
        // let mut cursor = Cursor::new(&buf[..response_len]);
        // let mut message = Message::from_bytes(&mut cursor).unwrap();

        // if message.header.num_answers == 1 {
        //     return Ok(message.answers.remove(0));
        // }

        // if message.header.num_additionals > 0 {
        //     if let Some(additional) = message
        //         .additionals
        //         .iter()
        //         .find(|p| matches!(p.data, RecordData::A(_)))
        //     {
        //         let RecordData::A(data) = additional.data else {
        //             return Err(DnsError::ServerFailure("Don't know how to continue".to_owned()))
        //         };
        //         nameserver = Ipv4Addr::from(data);
        //         continue;
        //     }
        // }

        // if message.header.num_authorities > 0 {
        //     let authority = message.authorities.first().ok_or(DnsError::ServerFailure(
        //         "Bad response from authority".to_owned(),
        //     ))?;
        //     let RecordData::Ns(ref name) = authority.data else {
        //         return Err(DnsError::ServerFailure("Non NS in authorities".to_owned()))
        //     };

        //     let question = Question::new(&name.get_full(), RecordType::A)?;

        //     let temp = resolve(sock, question, Arc::clone(&cache)).await?;
        //     let RecordData::A(data) = temp.data else {
        //         return Err(DnsError::ServerFailure("Failed to find authority".to_owned()))
        //     };

        //     nameserver = Ipv4Addr::from(data);
        //     continue;
        // }

        // return Err(DnsError::ServerFailure(
        //     "This should be unreachable".to_owned(),
        // ));
    }
}
