use std::io::Cursor;

use bytes::Buf;

use crate::{DnsError, Name, Networkable, RecordType};

#[derive(Debug, Clone)]
pub enum RecordData {
    A([u8; 4]),
    Ns(Name),
    Cname(Name),
    Mx { priority: u16, exchange: String },
    Txt(String),
    Aaaa([u8; 16]),
    Other,
}

impl RecordData {
    pub fn from_bytes(
        type_: RecordType,
        rd_length: u16,
        bytes: &mut Cursor<&[u8]>,
    ) -> Result<Self, DnsError> {
        match type_ {
            RecordType::A => Ok(Self::A(bytes.get_u32().to_be_bytes())),
            RecordType::Ns => Ok(Self::Ns(Name::from_bytes(bytes).unwrap())),
            RecordType::Cname => Ok(Self::Cname(Name::from_bytes(bytes).unwrap())),
            RecordType::Mx => Err(DnsError::NotImplemented),
            RecordType::Txt => Err(DnsError::NotImplemented),

            RecordType::Aaaa => {
                let result = Ok(Self::Aaaa(bytes.take(16).chunk().try_into().unwrap()));
                bytes.advance(16);
                result
            }

            other => {
                println!("Received record data of unknown type: {:?}", other);
                bytes.advance(rd_length as usize);
                Ok(Self::Other)
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::A(data) => data.to_vec(),
            Self::Ns(data) => data.to_bytes().to_vec(),
            Self::Cname(data) => data.to_bytes().to_vec(),
            Self::Mx { priority, exchange } => todo!(),
            Self::Txt(data) => data.as_bytes().to_vec(),
            Self::Aaaa(data) => data.to_vec(),
            Self::Other => Vec::new(),
        }
    }
}
