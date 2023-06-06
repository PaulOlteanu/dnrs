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
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_question(&mut self, question: Question) {
        self.header.qd_count += 1;
        self.questions.push(question);
    }
}

impl Networkable for Packet {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        let header = Header::from_bytes(bytes)?;

        let mut questions = Vec::new();
        for _ in 0..header.qd_count {
            questions.push(Question::from_bytes(bytes)?);
        }

        let mut answers = Vec::new();
        for _ in 0..header.an_count {
            answers.push(Record::from_bytes(bytes)?);
        }

        let mut authorities = Vec::new();
        for _ in 0..header.ns_count {
            authorities.push(Record::from_bytes(bytes)?);
        }

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
