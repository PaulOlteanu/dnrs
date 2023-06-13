use std::io::Cursor;

use bytes::Buf;
use num_traits::cast::{FromPrimitive, ToPrimitive};

use super::{Name, Networkable};
use crate::{RecordType, DnsError};

#[derive(Debug, Clone)]
pub struct Question {
    pub name: Name,
    pub type_: RecordType,
    pub class: u16,
}

impl Question {
    pub fn new(name: &str, type_: RecordType) -> Result<Self, DnsError> {
        if name.len() > 253 {
            return Err(DnsError::FormatError);
        }

        Ok(Self {
            name: Name::new(name),
            type_,
            class: 1,
        })
    }
}

impl Networkable for Question {
    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();

        ret.extend_from_slice(&self.name.to_bytes());
        ret.extend_from_slice(&self.type_.to_u16().unwrap().to_be_bytes());
        ret.extend_from_slice(&self.class.to_be_bytes());

        ret
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
        let name = Name::from_bytes(bytes).unwrap();

        let type_ = bytes.get_u16();
        let type_ = RecordType::from_u16(type_).ok_or(DnsError::FormatError)?;

        let class = bytes.get_u16();

        Ok(Self { name, type_, class })
    }
}
