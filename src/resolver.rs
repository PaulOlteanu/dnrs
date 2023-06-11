use std::net::Ipv4Addr;
use std::{io::Cursor, sync::Arc};

use async_recursion::async_recursion;
use dnrs::dns::{Header, Name, Networkable, Packet, Question, RecordData, RecordType};

use std::net::SocketAddr;

use tokio::net::UdpSocket;

use crate::error::DnrsError;

pub async fn run(ip: &str, port: u16) -> Result<(), DnrsError> {
    let sock = UdpSocket::bind((ip, port))
        .await
        .expect("Couldn't run server");

    let sock = Arc::new(sock);

    loop {
        let sock = Arc::clone(&sock);
        let mut buf = [0; 1024];
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("Received data");

        tokio::spawn(async move { handle_request(sock, addr, &buf[0..len]).await });
    }
}

async fn handle_request(sock: Arc<UdpSocket>, addr: SocketAddr, data: &[u8]) {
    let mut packet = Packet::from_bytes(&mut Cursor::new(data)).unwrap();

    if packet.header.flags.qr() {
        println!("Error");
    }

    // Validate request
    if packet.header.qd_count != 1 || packet.header.flags.opcode() != 0 {
        println!("Received unimplemented request");
        let mut response = Packet::new();
        response.header.flags.set_qr(true);
        response.header.flags.set_rcode(4);
        let buf = response.to_bytes();
        let _ = sock.send_to(&buf, addr).await;
    }

    // Request
    let sock = UdpSocket::bind(("0.0.0.0", 0))
        .await
        .expect("Couldn't couldn't create socket for request");

    let question = packet.questions.remove(0);
    let result = resolve(&sock, question).await;

    println!("Result: {:?}", result);
}

#[async_recursion]
async fn resolve(sock: &UdpSocket, question: Question) -> Result<Packet, ()> {
    let mut query = Packet::new();
    query.add_question(question);

    let mut nameserver = Ipv4Addr::new(198, 41, 0, 4);
    let mut buf = [0; 1024];

    loop {
        sock.send_to(&query.to_bytes(), (nameserver, 53))
            .await
            .unwrap();

        let response_len = sock.recv(&mut buf).await.unwrap();
        let mut cursor = Cursor::new(&buf[..response_len]);
        let packet = Packet::from_bytes(&mut cursor).unwrap();

        if packet.header.an_count == 1 {
            println!("Received answer");
            return Ok(packet);
        }

        if packet.header.ar_count > 0 {
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

        if packet.header.ns_count > 0 {
            let authority = packet.authorities.first().ok_or(())?;
            let RecordData::Ns(ref data) = authority.data else {
                return Err(())
            };

            let question = Question::new(&data.0, RecordType::A)?;

            let temp = resolve(sock, question).await?;
            let RecordData::A(data) = temp.answers.first().ok_or(())?.data else {
                return Err(())
            };

            nameserver = Ipv4Addr::from(data);
            continue;
        }

        return Err(());
    }
}
