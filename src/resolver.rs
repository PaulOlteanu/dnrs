use std::collections::HashMap;
use std::io::Cursor;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use dnrs::{
    DnsError, Flags, Header, Message, Name, Networkable, Question, RecordData, RecordType,
    ResourceRecord,
};
use tokio::net::UdpSocket;

use crate::util::set_response_flags;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub class: u16,
    pub type_: RecordType,
    pub name: Name,
}

impl CacheKey {
    pub fn new(class: u16, type_: RecordType, name: Name) -> Self {
        Self { class, type_, name }
    }
}

impl From<ResourceRecord> for CacheKey {
    fn from(value: ResourceRecord) -> Self {
        Self {
            class: value.class,
            type_: value.type_,
            name: value.name,
        }
    }
}

impl From<&ResourceRecord> for CacheKey {
    fn from(value: &ResourceRecord) -> Self {
        Self {
            class: value.class,
            type_: value.type_,
            name: value.name.clone(),
        }
    }
}

pub async fn run(ip: &str, port: u16) {
    let sock = UdpSocket::bind((ip, port))
        .await
        .expect("Couldn't run server");

    let cache = Arc::new(Mutex::new(HashMap::<CacheKey, ResourceRecord>::new()));

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

async fn handle_request(
    cache: Arc<Mutex<HashMap<CacheKey, ResourceRecord>>>,
    data: &[u8],
) -> Option<Message> {
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

    let key = CacheKey::new(question.class, question.type_, question.name.clone());

    let cached = {
        let cache = cache.lock().unwrap();
        cache.get(&key).map(Clone::clone)
    };

    if let Some(record) = cached {
        let flags = set_response_flags(request.header.flags);
        let header = Header::new(request.header.id, flags);

        let mut response = Message::new(header);
        response.add_question(question);
        response.add_answer(record.clone());

        println!(
            "Responding from cache for {:?}: {}",
            record.type_, record.name
        );

        Some(response)
    } else {
        if !request.header.flags.rd() {
            // TODO: Return no records
        }

        // Request
        let sub_sock = UdpSocket::bind(("0.0.0.0", 0))
            .await
            .or(Err(DnsError::ServerFailure(
                "Couldn't create socket to begin resolution".to_owned(),
            )))
            .unwrap();

        let result = resolve(&sub_sock, question.clone()).await;

        if let Ok(record) = result {
            {
                let mut cache = cache.lock().unwrap();
                cache.insert(CacheKey::from(&record), record.clone());
            }

            let flags = set_response_flags(request.header.flags);
            let header = Header::new(request.header.id, flags);

            let mut response = Message::new(header);
            response.add_question(question);
            response.add_answer(record.clone());

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
}

#[async_recursion]
async fn resolve(sock: &UdpSocket, question: Question) -> Result<ResourceRecord, DnsError> {
    let flags = Flags::default();
    let id = rand::random::<u16>();
    let header = Header::new(id, flags);
    let mut query = Message::new(header);
    query.questions.push(question);

    let mut nameserver = Ipv4Addr::new(198, 41, 0, 4);
    let mut buf = [0; 1024];

    loop {
        sock.send_to(&query.to_bytes(), (nameserver, 53))
            .await
            .unwrap();

        // TODO: Need to do validation of this message
        let response_len = sock.recv(&mut buf).await?;
        let mut cursor = Cursor::new(&buf[..response_len]);
        let mut message = Message::from_bytes(&mut cursor).unwrap();

        if message.header.num_answers == 1 {
            return Ok(message.answers.remove(0));
        }

        if message.header.num_additionals > 0 {
            if let Some(additional) = message
                .additionals
                .iter()
                .find(|p| matches!(p.data, RecordData::A(_)))
            {
                let RecordData::A(data) = additional.data else {
                    return Err(DnsError::ServerFailure("Don't know how to continue".to_owned()))
                };
                nameserver = Ipv4Addr::from(data);
                continue;
            }
        }

        if message.header.num_authorities > 0 {
            let authority = message.authorities.first().ok_or(DnsError::ServerFailure(
                "Bad response from authority".to_owned(),
            ))?;
            let RecordData::Ns(ref data) = authority.data else {
                return Err(DnsError::ServerFailure("Non NS in authorities".to_owned()))
            };

            let question = Question::new(&data.0, RecordType::A)?;

            let temp = resolve(sock, question).await?;
            let RecordData::A(data) = temp.data else {
                return Err(DnsError::ServerFailure("Failed to find authority".to_owned()))
            };

            nameserver = Ipv4Addr::from(data);
            continue;
        }

        return Err(DnsError::ServerFailure(
            "This should be unreachable".to_owned(),
        ));
    }
}
