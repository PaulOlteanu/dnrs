use std::io::{Cursor, Read};

use bytes::Buf;
use num_traits::cast::{FromPrimitive, ToPrimitive};

use super::{Name, Networkable, RecordType};

#[derive(Debug)]
pub struct Record {
    pub name: Name,
    pub type_: RecordType,
    pub class: u16,
    pub ttl: i32,
    pub rd_length: u16,
    pub data: Vec<u8>,
}

impl Networkable for Record {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend_from_slice(&self.name.to_bytes());
        ret.extend_from_slice(&self.type_.to_i16().unwrap().to_be_bytes());
        ret.extend_from_slice(&self.class.to_be_bytes());
        ret.extend_from_slice(&self.ttl.to_be_bytes());
        ret.extend_from_slice(&self.rd_length.to_be_bytes());
        ret.extend_from_slice(&self.data);

        ret
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        let name = Name::from_bytes(bytes)?;

        let type_ = bytes.get_u16();
        let type_ = RecordType::from_u16(type_).ok_or(())?;

        let class = bytes.get_u16();

        let ttl = bytes.get_i32();

        let rd_length = bytes.get_u16();

        let mut data = Vec::new();
        std::io::Read::take(bytes, rd_length as u64)
            .read_to_end(&mut data)
            .or(Err(()))?;

        Ok(Self {
            name,
            type_,
            class,
            ttl,
            rd_length,
            data,
        })
    }
}
