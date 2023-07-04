// use std::io::Cursor;

// use dnrs::{Flags, Header, Message, Name, Networkable, Question};
// use tokio::net::UdpSocket;

mod resolver;
mod util;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("trace")
        .pretty()
        .init();
    resolver::run("0.0.0.0", 3053).await;
}

// async fn query_resolver() {
//     let mut flags = Flags::default();
//     flags.set_rcode(1);
//     flags.set_rd(false);
//     let header = Header::new(323, flags);
//     let mut message = Message::new(header);
//     let question = Question::new(Name::new("ns1.google.ca"), dnrs::RecordType::A);
//     message.add_question(question);

//     let sock = UdpSocket::bind(("0.0.0.0", 3234)).await.unwrap();
//     let buf = message.to_bytes();
//     sock.send_to(&buf, ("192.5.6.30", 53)).await.unwrap();
//     let mut buf = [0; 1024];
//     let (recieved, _) = sock.recv_from(&mut buf).await.unwrap();
//     let mut cursor = Cursor::new(&buf[0..recieved]);
//     let response = Message::from_bytes(&mut cursor).unwrap();
//     println!("Response: {:#?}", response);
// }
