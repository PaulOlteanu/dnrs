use std::io::Cursor;

use super::{Header, Networkable, Question, Record};

#[derive(Debug)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub additionals: Vec<Record>,
}

impl Networkable for Packet {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        println!("Parsing header");
        let header = Header::from_bytes(bytes)?;

        println!("Parsing {} questions", header.qd_count);
        let mut questions = Vec::new();
        for _ in 0..header.qd_count {
            questions.push(Question::from_bytes(bytes)?);
        }

        println!("Parsing {} answers", header.an_count);
        let mut answers = Vec::new();
        for _ in 0..header.an_count {
            answers.push(Record::from_bytes(bytes)?);
        }

        println!("Parsing {} authorities", header.ns_count);
        let mut authorities = Vec::new();
        for _ in 0..header.ns_count {
            authorities.push(Record::from_bytes(bytes)?);
        }

        println!("Parsing {} additionals", header.ar_count);
        let mut additionals = Vec::new();
        for _ in 0..header.ar_count {
            additionals.push(Record::from_bytes(bytes)?);
        }

        Ok(Self {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
    }
}
