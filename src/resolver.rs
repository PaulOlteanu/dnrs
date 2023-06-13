use std::collections::HashMap;
use std::io::Cursor;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use async_recursion::async_recursion;
use dnrs::dns::{
    Flags, Header, Message, Name, Networkable, Question, RecordData, RecordType, ResourceRecord,
};
use tokio::net::UdpSocket;

use crate::error::DnrsError;
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

pub async fn run(ip: &str, port: u16) -> Result<(), DnrsError> {
    let sock = UdpSocket::bind((ip, port))
        .await
        .expect("Couldn't run server");

    let cache = Arc::new(Mutex::new(HashMap::<CacheKey, ResourceRecord>::new()));

    let sock = Arc::new(sock);

    loop {
        let sock = Arc::clone(&sock);
        let cache = Arc::clone(&cache);
        let mut buf = [0; 1024];
        let (len, addr) = sock.recv_from(&mut buf).await?;

        tokio::spawn(async move { handle_request(cache, sock, addr, &buf[0..len]).await });
    }
}

async fn handle_request(
    cache: Arc<Mutex<HashMap<CacheKey, ResourceRecord>>>,
    sock: Arc<UdpSocket>,
    addr: SocketAddr,
    data: &[u8],
) {
    let mut request = Message::from_bytes(&mut Cursor::new(data)).unwrap();

    if request.header.flags.qr() {
        println!("Error");
    }

    // Validate request
    if request.header.num_questions != 1 || request.header.flags.opcode() != 0 {
        println!("Received unimplemented request");
        let mut flags = set_response_flags(request.header.flags);
        flags.set_rcode(4);

        let header = Header::new(request.header.id, flags);
        let response = Message::new(header);
        let buf = response.to_bytes();
        sock.send_to(&buf, addr).await.ok();
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

        let response = response.to_bytes();

        sock.send_to(&response, addr).await.ok();
    } else {
        // Request
        let sub_sock = UdpSocket::bind(("0.0.0.0", 0))
            .await
            .expect("Couldn't couldn't create socket for request");

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

            let response = response.to_bytes();
            sock.send_to(&response, addr).await.ok();
        } else {
            let mut flags = set_response_flags(request.header.flags);
            flags.set_rcode(2);

            let header = Header::new(request.header.id, flags);

            let mut response = Message::new(header);
            response.add_question(question);

            let err = response.to_bytes();
            sock.send_to(&err, addr).await.ok();
        }
    }
}

#[async_recursion]
async fn resolve(sock: &UdpSocket, question: Question) -> Result<ResourceRecord, ()> {
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

        let response_len = sock.recv(&mut buf).await.unwrap();
        let mut cursor = Cursor::new(&buf[..response_len]);
        let mut packet = Message::from_bytes(&mut cursor).unwrap();

        if packet.header.num_answers == 1 {
            return Ok(packet.answers.remove(0));
        }

        if packet.header.num_additionals > 0 {
            if let Some(additional) = packet
                .additionals
                .iter()
                .find(|p| matches!(p.data, RecordData::A(_)))
            {
                let RecordData::A(data) = additional.data else {
                    return Err(())
                };
                nameserver = Ipv4Addr::from(data);
                continue;
            }
        }

        if packet.header.num_authorities > 0 {
            let authority = packet.authorities.first().ok_or(())?;
            let RecordData::Ns(ref data) = authority.data else {
                return Err(())
            };

            let question = Question::new(&data.0, RecordType::A)?;

            let temp = resolve(sock, question).await?;
            let RecordData::A(data) = temp.data else {
                return Err(())
            };

            nameserver = Ipv4Addr::from(data);
            continue;
        }

        return Err(());
    }
}
