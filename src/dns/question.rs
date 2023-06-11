use std::io::Cursor;

use bytes::Buf;
use num_traits::cast::{FromPrimitive, ToPrimitive};

use super::{Name, Networkable, RecordType};

#[derive(Debug, Clone)]
pub struct Question {
    pub name: Name,
    pub type_: RecordType,
    pub class: u16,
}

impl Question {
    pub fn new(name: &str, type_: RecordType) -> Result<Self, ()> {
        if name.len() > 253 {
            return Err(());
        }

        Ok(Self {
            name: Name::new(name),
            type_,
            class: 1,
        })
    }
}

impl Networkable for Question {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();

        ret.extend_from_slice(&self.name.to_bytes());
        ret.extend_from_slice(&self.type_.to_u16().unwrap().to_be_bytes());
        ret.extend_from_slice(&self.class.to_be_bytes());

        ret
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        let name = Name::from_bytes(bytes).unwrap();

        let type_ = bytes.get_u16();
        let type_ = RecordType::from_u16(type_).unwrap();

        let class = bytes.get_u16();

        Ok(Self { name, type_, class })
    }
}
