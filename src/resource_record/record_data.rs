use std::io::Cursor;
use std::net::{Ipv4Addr, Ipv6Addr};

use bytes::Buf;

use crate::{DnsError, Name, Networkable, RecordType};

#[derive(Debug, Clone)]
pub enum RecordData {
    A(Ipv4Addr),
    Ns(Name),
    Cname(Name),
    Mx { priority: u16, exchange: String },
    Txt(String),
    Aaaa(Ipv6Addr),
    Other,
}

impl RecordData {
    pub fn from_bytes(
        type_: RecordType,
        rd_length: u16,
        bytes: &mut Cursor<&[u8]>,
    ) -> Result<Self, DnsError> {
        match type_ {
            RecordType::A => Ok(Self::A(bytes.get_u32().to_be_bytes().into())),
            RecordType::Ns => Ok(Self::Ns(Name::from_bytes(bytes).unwrap())),
            RecordType::Cname => Ok(Self::Cname(Name::from_bytes(bytes).unwrap())),
            RecordType::Mx => Err(DnsError::NotImplemented),
            RecordType::Txt => Err(DnsError::NotImplemented),
            RecordType::Aaaa => Ok(Self::Aaaa(bytes.get_u128().to_be_bytes().into())),

            other => {
                println!("Received record data of unknown type: {:?}", other);
                bytes.advance(rd_length as usize);
                Ok(Self::Other)
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::A(data) => u32::from(*data).to_be_bytes().to_vec(),
            Self::Ns(data) => data.to_bytes().to_vec(),
            Self::Cname(data) => data.to_bytes().to_vec(),
            Self::Mx { priority, exchange } => todo!(),
            Self::Txt(data) => data.as_bytes().to_vec(),
            Self::Aaaa(data) => u128::from(*data).to_be_bytes().to_vec(),
            Self::Other => Vec::new(),
        }
    }
}
