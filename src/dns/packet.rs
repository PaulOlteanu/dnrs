use std::io::Cursor;

use super::{Header, Networkable, Question, Record};

#[derive(Debug, Default)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub additionals: Vec<Record>,
}

impl Packet {
    pub fn new(header: Header) -> Self {
        Self {
            header,
            ..Default::default()
        }
    }
}

impl Networkable for Packet {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        response.extend_from_slice(&self.header.to_bytes());
        for question in self.questions.iter() {
            response.extend_from_slice(&question.to_bytes())
        }
        for record in self.answers.iter() {
            response.extend_from_slice(&record.to_bytes())
        }

        for record in self.authorities.iter() {
            response.extend_from_slice(&record.to_bytes())
        }

        for record in self.additionals.iter() {
            response.extend_from_slice(&record.to_bytes())
        }

        response
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        let header = Header::from_bytes(bytes).unwrap();

        let mut questions = Vec::new();
        for _ in 0..header.qd_count {
            questions.push(Question::from_bytes(bytes).unwrap());
        }

        let mut answers = Vec::new();
        for _ in 0..header.an_count {
            answers.push(Record::from_bytes(bytes).unwrap());
        }

        let mut authorities = Vec::new();
        for _ in 0..header.ns_count {
            authorities.push(Record::from_bytes(bytes).unwrap());
        }

        let mut additionals = Vec::new();
        for _ in 0..header.ar_count {
            additionals.push(Record::from_bytes(bytes).unwrap());
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
