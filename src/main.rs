use std::io::Cursor;
use std::net::UdpSocket;

use dnrs::dns::{Networkable, Packet, RecordType};

use crate::util::create_query;

mod util;

fn main() {
    let request = create_query("www.example.com", RecordType::A).unwrap();

    let sock = UdpSocket::bind("0.0.0.0:34254").unwrap();
    sock.send_to(&request, "8.8.8.8:53").unwrap();
    let mut buf = [0; 1024];
    let response_len = sock.recv(&mut buf).unwrap();

    let mut cursor = Cursor::new(&buf[..response_len]);

    let packet = Packet::from_bytes(&mut cursor).unwrap();

    println!("{:#?}", packet.answers);
}
