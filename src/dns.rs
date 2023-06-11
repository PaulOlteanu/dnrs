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
pub use record::{Record, RecordData};

#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RecordType {
    A = 1,
    Ns = 2,
    Md = 3,
    Mf = 4,
    Cname = 5,
    Soa = 6,
    Mb = 7,
    Mg = 8,
    Mr = 9,
    Null = 10,
    Wks = 11,
    Ptr = 12,
    Hinfo = 13,
    Minfo = 14,
    Mx = 15,
    Txt = 16,
    Aaaa = 28,
    Opt = 41,
}

pub trait Networkable: Sized {
    type Error;

    fn to_bytes(&self) -> Vec<u8>;

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error>;
}
