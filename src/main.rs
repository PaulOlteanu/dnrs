use std::io::Cursor;
use std::net::{Ipv4Addr, UdpSocket};

use dnrs::dns::{Header, Name, Networkable, Packet, Question, RecordData, RecordType};

use crate::util::create_query;

mod util;

fn main() {
    let sock = UdpSocket::bind("0.0.0.0:34254").unwrap();
    let result = resolve(&sock, "google.com", RecordType::A);

    println!("{:?}", result);
}

fn resolve(sock: &UdpSocket, domain: &str, type_: RecordType) -> Result<Packet, ()> {
    let request_buf = create_query(domain, type_)?;

    let mut nameserver = Ipv4Addr::new(198, 41, 0, 4);
    let mut buf = [0; 1024];

    loop {
        println!("Querying {nameserver} for {domain}");

        sock.send_to(&request_buf, (nameserver, 53)).or(Err(()))?;

        let response_len = sock.recv(&mut buf).unwrap();
        let mut cursor = Cursor::new(&buf[..response_len]);
        let packet = Packet::from_bytes(&mut cursor).unwrap();

        if packet.header.an_count > 0 {
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

            let temp = resolve(sock, &data.0, RecordType::A)?;
            let RecordData::A(data) = temp.answers.first().ok_or(())?.data else {
                return Err(())
            };

            nameserver = Ipv4Addr::from(data);
            continue;
        }

        return Err(());
    }
}
