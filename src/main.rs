use std::io::Cursor;

use dnrs::{Flags, Header, Message, Networkable, Question};
use tokio::net::UdpSocket;

mod resolver;
mod util;

#[tokio::main]
async fn main() {
    resolver::run("0.0.0.0", 3053).await;
}

async fn query_resolver() {
    let mut flags = Flags::default();
    flags.set_rcode(1);
    flags.set_rd(true);
    let header = Header::new(323, flags);
    let mut message = Message::new(header);
    let question = Question::new("google.com", dnrs::RecordType::A).unwrap();
    message.add_question(question);

    let sock = UdpSocket::bind(("0.0.0.0", 3234)).await.unwrap();
    let buf = message.to_bytes();
    sock.send_to(&buf, ("1.1.1.1", 53)).await.unwrap();
    let mut buf = [0; 1024];
    let (recieved, _) = sock.recv_from(&mut buf).await.unwrap();
    let mut cursor = Cursor::new(&buf[0..recieved]);
    let response = Message::from_bytes(&mut cursor).unwrap();
    println!("Response: {:?}", response);
}
