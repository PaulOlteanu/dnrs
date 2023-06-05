use std::io::Cursor;

use bitfield::bitfield;
use bytes::Buf;

use super::Networkable;

bitfield! {
  pub struct Flags(u16);
  impl Debug;
  u8;
  // query or response
  pub qr, set_qr: 15;
  // query type
  pub opcode, set_opcode: 14, 11;
  // authoritative answerer
  pub aa, set_aa: 10;
  // truncation
  pub tc, set_tc: 9;
  // recursion desired
  pub rd, set_rd: 8;
  // recursion available
  pub ra, set_ra: 7;
  // reserved
  pub z, set_z: 6, 4;
  // response code
  pub rcode, set_rcode: 3, 0;
}

impl Default for Flags {
    fn default() -> Self {
        Self(0)
    }
}

impl Networkable for Flags {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        if bytes.remaining() < 2 {
            return Err(());
        }

        let flags = bytes.get_u16();

        Ok(Self(flags))
    }
}

#[derive(Debug, Default)]
pub struct Header {
    pub id: u16,
    pub flags: Flags,
    // num questions
    pub qd_count: u16,
    // num answers
    pub an_count: u16,
    // num authorities
    pub ns_count: u16,
    // num additionals
    pub ar_count: u16,
}

impl Networkable for Header {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(12);
        ret.extend_from_slice(&self.id.to_be_bytes());
        ret.extend_from_slice(&self.flags.to_bytes());
        ret.extend_from_slice(&self.qd_count.to_be_bytes());
        ret.extend_from_slice(&self.an_count.to_be_bytes());
        ret.extend_from_slice(&self.ns_count.to_be_bytes());
        ret.extend_from_slice(&self.ar_count.to_be_bytes());

        ret
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        if bytes.remaining() < 12 {
            return Err(());
        }

        let id = bytes.get_u16();
        let flags = Flags::from_bytes(bytes)?;
        let qd_count = bytes.get_u16();
        let an_count = bytes.get_u16();
        let ns_count = bytes.get_u16();
        let ar_count = bytes.get_u16();

        Ok(Self {
            id,
            flags,
            qd_count,
            an_count,
            ns_count,
            ar_count,
        })
    }
}
