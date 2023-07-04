use std::io::Cursor;
use std::net::{Ipv4Addr, Ipv6Addr};

use bytes::Buf;
use enum_filter::enum_filter;
use tracing::warn;

use crate::{DnsError, Name, Networkable, RecordType};

#[enum_filter]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RecordData {
    A(Ipv4Addr),
    Ns(Name),
    Cname(Name),
    Soa {
        mname: Name,
        rname: Name,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    },
    Mx {
        priority: u16,
        exchange: String,
    },
    Txt(String),
    Aaaa(Ipv6Addr),
    Other,
}

impl RecordData {
    pub fn from_bytes(
        type_: RecordType,
        rd_length: u16,
        bytes: &mut Cursor<&[u8]>,
    ) -> Result<Self, DnsError> {
        match type_ {
            RecordType::A => Ok(Self::A(bytes.get_u32().to_be_bytes().into())),
            RecordType::Ns => Ok(Self::Ns(Name::from_bytes(bytes)?)),
            RecordType::Cname => Ok(Self::Cname(Name::from_bytes(bytes)?)),
            RecordType::Soa => Ok(Self::Soa {
                mname: Name::from_bytes(bytes)?,
                rname: Name::from_bytes(bytes)?,
                serial: bytes.get_u32(),
                refresh: bytes.get_u32(),
                retry: bytes.get_u32(),
                expire: bytes.get_u32(),
                minimum: bytes.get_u32(),
            }),
            RecordType::Mx => Err(DnsError::NotImplemented),
            RecordType::Txt => Err(DnsError::NotImplemented),
            RecordType::Aaaa => Ok(Self::Aaaa(bytes.get_u128().to_be_bytes().into())),

            record_type => {
                warn!(?record_type, "received unimplemented record data");
                bytes.advance(rd_length as usize);
                Ok(Self::Other)
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::A(data) => u32::from(*data).to_be_bytes().to_vec(),
            Self::Ns(data) => data.to_bytes().to_vec(),
            Self::Cname(data) => data.to_bytes().to_vec(),
            Self::Soa {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum,
            } => {
                let mut ret = Vec::new();
                ret.extend_from_slice(&mname.to_bytes());
                ret.extend_from_slice(&rname.to_bytes());
                ret.extend_from_slice(&serial.to_be_bytes());
                ret.extend_from_slice(&refresh.to_be_bytes());
                ret.extend_from_slice(&retry.to_be_bytes());
                ret.extend_from_slice(&expire.to_be_bytes());
                ret.extend_from_slice(&minimum.to_be_bytes());
                ret
            }
            Self::Mx { priority, exchange } => todo!(),
            Self::Txt(data) => data.as_bytes().to_vec(),
            Self::Aaaa(data) => u128::from(*data).to_be_bytes().to_vec(),
            Self::Other => Vec::new(),
        }
    }
}
