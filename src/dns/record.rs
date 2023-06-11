use std::io::Cursor;

use bytes::Buf;

use super::{Name, Networkable};

mod record_data;
pub use record_data::RecordData;

#[derive(Debug)]
pub struct Record {
    pub name: Name,
    // TODO: Maybe get rid of this
    pub type_: u16,
    pub class: u16,
    pub ttl: u32,
    pub rd_length: u16,
    pub data: RecordData,
}

impl Networkable for Record {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend_from_slice(&self.name.to_bytes());
        ret.extend_from_slice(&self.type_.to_be_bytes());
        ret.extend_from_slice(&self.class.to_be_bytes());
        ret.extend_from_slice(&self.ttl.to_be_bytes());
        ret.extend_from_slice(&self.rd_length.to_be_bytes());
        ret.extend_from_slice(&self.data.to_bytes());

        ret
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        let name = Name::from_bytes(bytes).unwrap();
        let type_ = bytes.get_u16();
        let class = bytes.get_u16();
        let ttl = bytes.get_u32();
        let rd_length = bytes.get_u16();

        let data = RecordData::from_bytes(type_, rd_length, bytes)?;

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
