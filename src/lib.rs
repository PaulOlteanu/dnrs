use std::io::Cursor;

mod error;
use bytes::Bytes;
pub use error::DnsError;

mod header;
pub use header::{Flags, Header};

mod name;
pub use name::Name;

mod message;
pub use message::Message;

mod question;
pub use question::Question;

mod resource_record;
pub use resource_record::{
    RecordData, RecordDataFilter, RecordDataMutRefFilter, RecordDataRefFilter, ResourceRecord,
};

mod record_type;
pub use record_type::RecordType;

pub trait Networkable: Sized {
    fn to_bytes(&self) -> Bytes;
    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError>;
}
