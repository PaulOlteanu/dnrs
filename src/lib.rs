use std::io::Cursor;

mod error;
use bytes::Bytes;
use enum_dispatch::enum_dispatch;
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
pub use resource_record::ResourceRecord;

mod record_type;
pub use record_type::RecordType;

// use crate::resource_record::{Aaaa, Cname, Mx, Ns, Soa, Txt, A};

// #[enum_dispatch]
pub trait ByteSer {
    fn to_bytes(&self) -> Bytes;
}
