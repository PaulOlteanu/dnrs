use std::io::Cursor;

use bytes::Buf;
use derivative::Derivative;
use tracing::instrument;

use super::{Name, Networkable};
use crate::{DnsError, RecordType};

mod record_data;
pub use record_data::RecordData;

#[derive(Derivative)]
#[derivative(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceRecord {
    pub name: Name,
    pub type_: RecordType,
    pub class: u16,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub ttl: u32,
    pub data: RecordData,
}

impl Networkable for ResourceRecord {
    #[instrument(level = "trace", skip_all)]
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

    #[instrument(level = "trace", skip_all)]
    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
        let name = Name::from_bytes(bytes).unwrap();
        let type_ = bytes.get_u16();
        let type_ = RecordType::from_int(type_).ok_or(DnsError::FormatError)?;
        let class = bytes.get_u16();
        let ttl = bytes.get_u32();
        let data_length = bytes.get_u16();

        let data = RecordData::from_bytes(type_, data_length, bytes).unwrap();

        Ok(Self {
            name,
            type_,
            class,
            ttl,
            data,
        })
    }
}
