use std::io::Cursor;

mod error;
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
pub use resource_record::{RecordData, ResourceRecord};

mod record_type;
pub use record_type::RecordType;

pub trait Networkable: Sized {
    type Error;

    fn to_bytes(&self) -> Vec<u8>;

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error>;
}
