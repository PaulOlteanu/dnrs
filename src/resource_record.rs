use std::io::Cursor;

use bytes::Buf;
use num_traits::FromPrimitive;

use super::{Name, Networkable};
use crate::{DnsError, RecordType};

mod record_data;
pub use record_data::RecordData;

#[derive(Debug, Clone)]
pub struct ResourceRecord {
    pub name: Name,
    pub type_: RecordType,
    pub class: u16,
    pub ttl: u32,
    pub rd_length: u16,
    pub data: RecordData,
}

impl Networkable for ResourceRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend_from_slice(&self.name.to_bytes());
        ret.extend_from_slice(&(self.type_ as u16).to_be_bytes());
        ret.extend_from_slice(&self.class.to_be_bytes());
        ret.extend_from_slice(&self.ttl.to_be_bytes());
        let data = self.data.to_bytes();
        let rd_length = data.len() as u16;
        ret.extend_from_slice(&rd_length.to_be_bytes());
        ret.extend_from_slice(&data);

        ret
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
        let name = Name::from_bytes(bytes).unwrap();
        let type_ = bytes.get_u16();
        let type_ = FromPrimitive::from_u16(type_)
            .ok_or(DnsError::FormatError)
            .unwrap();
        let class = bytes.get_u16();
        let ttl = bytes.get_u32();
        let rd_length = bytes.get_u16();

        // let data = RecordData::from_bytes(type_, rd_length, bytes)?;
        let data = RecordData::from_bytes(type_, rd_length, bytes).unwrap();

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
