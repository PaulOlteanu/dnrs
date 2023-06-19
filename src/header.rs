use std::io::Cursor;

use bitfield::bitfield;
use bytes::Buf;
use tracing::{instrument, warn};

use super::Networkable;
use crate::DnsError;

bitfield! {
    #[derive(Clone, Copy, Default)]
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
    pub z, set_z: 6;
    pub ad, set_ad: 5;
    pub cd, set_cd: 4;
    // response code
    pub rcode, set_rcode: 3, 0;
}

impl Networkable for Flags {
    #[instrument(level = "trace", skip_all)]
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    #[instrument(level = "trace", skip_all)]
    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
        if bytes.remaining() < 2 {
            return Err(DnsError::FormatError);
        }

        let flags = bytes.get_u16();

        Ok(Self(flags))
    }
}

#[derive(Debug, Default)]
pub struct Header {
    pub id: u16,
    pub flags: Flags,
    pub num_questions: u16,
    pub num_answers: u16,
    pub num_authorities: u16,
    pub num_additionals: u16,
}

impl Header {
    pub fn new(id: u16, flags: Flags) -> Self {
        Self {
            id,
            flags,
            ..Default::default()
        }
    }
}

impl Networkable for Header {
    #[instrument(level = "trace", skip_all)]
    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(12);
        ret.extend_from_slice(&self.id.to_be_bytes());
        ret.extend_from_slice(&self.flags.to_bytes());
        ret.extend_from_slice(&self.num_questions.to_be_bytes());
        ret.extend_from_slice(&self.num_answers.to_be_bytes());
        ret.extend_from_slice(&self.num_authorities.to_be_bytes());
        ret.extend_from_slice(&self.num_additionals.to_be_bytes());

        ret
    }

    #[instrument(level = "trace", skip_all)]
    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
        if bytes.remaining() < 12 {
            warn!("insufficient remaining bytes");
            return Err(DnsError::FormatError);
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
            num_questions: qd_count,
            num_answers: an_count,
            num_authorities: ns_count,
            num_additionals: ar_count,
        })
    }
}
