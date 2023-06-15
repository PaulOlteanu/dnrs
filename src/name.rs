use std::fmt::Display;
use std::io::Cursor;

use bytes::Buf;
use itertools::Itertools;

use super::Networkable;
use crate::DnsError;

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub struct Label(pub String);

// TODO: Need to implement hash, partialeq, and eq on my own
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Name {
    /// This is ordered from right to left in a domain name
    /// For example: www.example.com becomes [com, example, www]
    pub name: Vec<String>,
    /// This is an index into name
    /// So level 0 would be com and level 2 would be www.example.com
    pub level: usize,
}

impl Name {
    // TODO: Checking on the length
    pub fn new(name: &str) -> Self {
        let name: Vec<String> = name.split('.').rev().map(|l| l.to_owned()).collect();
        let level = name.len() - 1;
        Self { name, level }
    }

    pub fn get_current_subdomain(&self) -> String {
        self.name[0..=self.level].iter().rev().cloned().join(".")
    }

    pub fn get_full(&self) -> String {
        self.name.iter().rev().cloned().join(".")
    }

    pub fn increase_level(&mut self) {
        if self.level < self.name.len() - 1 {
            self.level += 1;
        }
    }

    pub fn decrease_level(&mut self) {
        if self.level > 0 {
            self.level -= 1;
        }
    }

    // Get larger and larger subdomains
    // Eg www.google.com -> [com, google.com, www.google.com]
    pub fn iter_subdomains(&self) -> Vec<String> {
        self.name.iter().fold(Vec::new(), |mut acc, label| {
            if let Some(prev) = acc.last() {
                acc.push(format!("{}.{}", label, prev));
                acc
            } else {
                acc.push(label.to_owned());
                acc
            }
        })
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.get_full())
    }
}

impl Networkable for Name {
    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();

        for section in self.name.iter().rev() {
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
                parts.push(n.get_full());
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
    fn creates_successfully() {
        let name = Name::new("www.google.com");

        assert!(name.level == 2);
        assert!(name.get_full() == "www.google.com");
        assert!(name.get_current_subdomain() == "www.google.com");
    }

    #[test]
    fn handles_levels() {
        let mut name = Name::new("www.google.com");

        assert!(name.get_current_subdomain() == "www.google.com");
        name.decrease_level();
        assert!(name.get_current_subdomain() == "google.com");
        name.decrease_level();
        assert!(name.get_current_subdomain() == "com");
        name.decrease_level();
        assert!(name.get_current_subdomain() == "com");
        name.increase_level();
        assert!(name.get_current_subdomain() == "google.com");
        name.increase_level();
        assert!(name.get_current_subdomain() == "www.google.com");
        name.increase_level();
        assert!(name.get_current_subdomain() == "www.google.com");
    }
}
