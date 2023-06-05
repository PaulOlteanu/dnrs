use std::io::Cursor;

mod header;
pub use header::{Flags, Header};

mod name;
pub use name::Name;

mod packet;
pub use packet::Packet;

mod question;
pub use question::Question;

mod record;
use num_derive::{FromPrimitive, ToPrimitive};
pub use record::Record;

#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum RecordType {
    A = 1,
    NS = 2,
    MD = 3,
    MF = 4,
    CNAME = 5,
    SOA = 6,
    MB = 7,
    MG = 8,
    MR = 9,
    NULL = 10,
    WKS = 11,
    PTR = 12,
    HINFO = 13,
    MINFO = 14,
    MX = 15,
    TXT = 16,
}

pub trait Networkable: Sized {
    type Error;

    fn to_bytes(&self) -> Vec<u8>;

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error>;
}
