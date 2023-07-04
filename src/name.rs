use std::fmt::Display;
use std::io::Cursor;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::Networkable;
use crate::DnsError;

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub struct Label(pub String);

// TODO: Need to implement hash, partialeq, and eq on my own
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Name {
    /// This is the domain name
    /// E.g. www.google.com
    pub name: String,

    /// This is a vector of all the indices where a label starts
    /// E.g. www.google.com would have a split_indices of [0, 4, 11]
    pub split_indices: Vec<usize>,
}

impl Name {
    // TODO: Checking on the length
    pub fn new(name: &str) -> Self {
        let mut split_indices = vec![0];
        split_indices.extend(name.match_indices('.').filter_map(|(i, _)| {
            if i == name.len() - 1 {
                None
            } else {
                Some(i + 1)
            }
        }));

        Self {
            name: name.to_owned(),
            split_indices,
        }
    }

    // TODO: Rename this
    /// Get larger and larger subdomains
    /// Eg www.google.com -> [com, google.com, www.google.com]
    pub fn iter_subdomains(&self) -> impl Iterator<Item = String> + '_ {
        self.split_indices
            .iter()
            .rev()
            .map(|i| self.name[*i..].to_owned())
    }

    pub fn matching_level(&self, other: &Name) -> usize {
        let a = self.iter_subdomains();
        let b = other.iter_subdomains();

        a.zip(b)
            .enumerate()
            .map_while(|(i, (a, b))| if a == b { Some(i + 1) } else { None })
            .last()
            .unwrap_or(0)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl Networkable for Name {
    fn to_bytes(&self) -> Bytes {
        let mut ret = BytesMut::new();

        for section in self.name.split('.') {
            ret.put_u8(section.len() as u8);
            ret.extend_from_slice(section.as_bytes());
        }

        ret.put_u8(0);

        ret.into()
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
                parts.push(n.name);
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

        Ok(Self::new(&name))
    }
}

#[cfg(test)]
mod tests {
    use crate::Name;

    #[test]
    fn generates_subdomain_iter() {
        let name = Name::new("www.google.com");
        let fact: Vec<String> = name.iter_subdomains().collect();
        assert_eq!(fact, ["com", "google.com", "www.google.com"]);
    }

    #[test]
    fn calculates_level() {
        let name1 = Name::new("asdf.google.com");
        let name2 = Name::new("jkl.google.com");

        assert_eq!(2, name1.matching_level(&name2));
    }
}
