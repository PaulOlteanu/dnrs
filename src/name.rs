use std::fmt::Display;
use std::io::Cursor;

use bytes::Buf;

use crate::DnsError;

use super::Networkable;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Name(pub String);

impl Name {
    // TODO: Checking on the length
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Networkable for Name {
    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();

        for section in self.0.split('.') {
            ret.push(section.len() as u8);
            ret.extend_from_slice(section.as_bytes());
        }

        ret.push(0);

        ret
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, DnsError> {
        let mut parts: Vec<String> = Vec::new();
        loop {
            let len = bytes.get_u8() as usize;
            if len == 0 {
                break;
            }

            if (len & 0b1100_0000) >> 6 == 0b11 {
                // Compressed
                let pointer = (((len & 0b0011_1111) as u16) << 8) | (bytes.get_u8() as u16);
                let position = bytes.position();
                bytes.set_position(pointer as u64);
                let n = Self::from_bytes(bytes)?;
                bytes.set_position(position);
                parts.push(n.0);
                break;
            } else {
                // Uncompressed
                if bytes.remaining() < len {
                    return Err(DnsError::FormatError);
                }

                let chars = bytes.copy_to_bytes(len);
                let s = std::str::from_utf8(&chars).or(Err(DnsError::FormatError))?;
                parts.push(s.to_owned());
            }
        }

        let name = parts.join(".");

        Ok(Self(name))
    }
}
