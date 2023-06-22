use bytes::{BufMut, Bytes, BytesMut};
use enum_dispatch::enum_dispatch;
use std::io::Cursor;

use bytes::Buf;
use derivative::Derivative;
use tracing::instrument;

use super::{ByteSer, Name};
use crate::{DnsError, RecordType};

mod a;
pub use a::A;

// mod aaaa;
// pub use aaaa::Aaaa;

// mod cname;
// pub use cname::Cname;

// mod mx;
// pub use mx::Mx;

// mod ns;
// pub use ns::Ns;

// mod soa;
// pub use soa::Soa;

// mod txt;
// pub use txt::Txt;

// #[enum_dispatch(ByteSer)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ResourceRecord {
    A,
    Aaaa,
    Cname,
    Mx,
    Ns,
    Soa,
    Txt,
}

// impl Networkable for ResourceRecord {
//     #[instrument(level = "trace", skip_all)]
//     fn to_bytes(&self) -> Bytes {
//         let mut ret = BytesMut::new();
//         ret.extend_from_slice(&self.name.to_bytes());
//         ret.put_u16(self.type_.to_int());
//         ret.extend_from_slice(&self.class.to_be_bytes());
//         ret.extend_from_slice(&self.ttl.to_be_bytes());
//         let data = self.data.to_bytes();
//         let rd_length = data.len() as u16;
//         ret.extend_from_slice(&rd_length.to_be_bytes());
//         ret.extend_from_slice(&data);

//         ret.into()
//     }

// #[instrument(level = "trace", skip_all)]
// fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
//     let name = Name::from_bytes(bytes).unwrap();
//     let type_ = bytes.get_u16();
//     let type_ = RecordType::from_int(type_).ok_or(DnsError::FormatError)?;
//     let class = bytes.get_u16();
//     let ttl = bytes.get_u32();
//     let data_length = bytes.get_u16();

//     let data = RecordData::from_bytes(type_, data_length, bytes).unwrap();

//     Ok(Self {
//         name,
//         type_,
//         class,
//         ttl,
//         data,
//     })
// }
// }
