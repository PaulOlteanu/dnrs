use std::io::Cursor;

use bytes::{Buf, Bytes, BytesMut};
use tracing::instrument;

use super::{Name, Networkable};
use crate::{DnsError, RecordType};

#[derive(Debug, Clone)]
pub struct Question {
    pub name: Name,
    pub type_: RecordType,
    pub class: u16,
}

impl Question {
    pub fn new(name: Name, type_: RecordType) -> Self {
        Self {
            name,
            type_,
            class: 1,
        }
    }
}

impl Networkable for Question {
    #[instrument(level = "trace", skip_all)]
    fn to_bytes(&self) -> Bytes {
        let mut ret = BytesMut::new();

        ret.extend_from_slice(&self.name.to_bytes());
        ret.extend_from_slice(&(self.type_ as u16).to_be_bytes());
        ret.extend_from_slice(&self.class.to_be_bytes());

        ret.into()
    }

    #[instrument(level = "trace", skip_all)]
    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
        let name = Name::from_bytes(bytes).unwrap();

        let type_ = bytes.get_u16();
        let type_ = RecordType::from_int(type_).ok_or(DnsError::FormatError)?;

        let class = bytes.get_u16();

        Ok(Self { name, type_, class })
    }
}
