use bytes::{Bytes, BytesMut};
use std::io::Cursor;

use tracing::instrument;

use super::{ByteSer, Header, Question, ResourceRecord};
use crate::DnsError;

#[derive(Debug, Default)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authorities: Vec<ResourceRecord>,
    pub additionals: Vec<ResourceRecord>,
}

impl Message {
    pub fn new(header: Header) -> Self {
        Self {
            header,
            ..Default::default()
        }
    }

    pub fn add_question(&mut self, question: Question) {
        self.header.num_questions += 1;
        self.questions.push(question)
    }
    pub fn add_answer(&mut self, answer: ResourceRecord) {
        self.header.num_answers += 1;
        self.answers.push(answer)
    }

    pub fn add_authority(&mut self, answer: ResourceRecord) {
        self.header.num_authorities += 1;
        self.authorities.push(answer)
    }

    pub fn add_additional(&mut self, answer: ResourceRecord) {
        self.header.num_additionals += 1;
        self.additionals.push(answer)
    }
}

impl ByteSer for Message {
    #[instrument(level = "debug", skip_all)]
    fn to_bytes(&self) -> Bytes {
        let mut response = BytesMut::new();
        response.extend_from_slice(&self.header.to_bytes());

        for question in self.questions.iter() {
            response.extend_from_slice(&question.to_bytes())
        }

        // for record in self.answers.iter() {
        //     response.extend_from_slice(&record.to_bytes())
        // }

        // for record in self.authorities.iter() {
        //     response.extend_from_slice(&record.to_bytes())
        // }

        // for record in self.additionals.iter() {
        //     response.extend_from_slice(&record.to_bytes())
        // }

        response.into()
    }
}

// #[instrument(level = "debug", skip_all)]
// fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
//     let header = Header::from_bytes(bytes).unwrap();

//     let mut questions = Vec::new();
//     for _ in 0..header.num_questions {
//         questions.push(Question::from_bytes(bytes).unwrap());
//     }

//     let mut answers = Vec::new();
//     for _ in 0..header.num_answers {
//         answers.push(ResourceRecord::from_bytes(bytes).unwrap());
//     }

//     let mut authorities = Vec::new();
//     for _ in 0..header.num_authorities {
//         authorities.push(ResourceRecord::from_bytes(bytes).unwrap());
//     }

//     let mut additionals = Vec::new();
//     for _ in 0..header.num_additionals {
//         additionals.push(ResourceRecord::from_bytes(bytes).unwrap());
//     }

//     Ok(Self {
//         header,
//         questions,
//         answers,
//         authorities,
//         additionals,
//     })
// }
