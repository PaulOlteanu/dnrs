use std::io::{Cursor, Read};

use bytes::Buf;

use super::{Name, Networkable};

mod a_data;
use a_data::AData;

mod c_name_data;
use c_name_data::CNameData;

#[derive(Debug)]
pub enum RecordData {
    A(AData),
    Ns,
    CName(CNameData),
    Mx,
    Txt,
    // Aaaa(AaaaData),
}

impl RecordData {
    pub fn from_data(type_: u16, data: &[u8]) -> Result<Self, ()> {
        match type_ {
            1 => Ok(Self::A(AData::from_data(data)?)),
            2 => todo!(), // NS
            5 => Ok(Self::CName(CNameData::from_data(data)?)),
            15 => todo!(), // MX
            16 => todo!(), // TXT
            _ => unimplemented!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::A(data) => data.to_data(),
            Self::Ns => todo!(),
            Self::CName(data) => data.to_data(),
            Self::Mx => todo!(),
            Self::Txt => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct Record {
    pub name: Name,
    // TODO: Maybe can get rid of this
    pub type_: u16,
    pub class: u16,
    pub ttl: i32,
    pub rd_length: u16,
    pub data: RecordData,
}

impl Networkable for Record {
    type Error = ();

    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        ret.extend_from_slice(&self.name.to_bytes());
        ret.extend_from_slice(&self.type_.to_be_bytes());
        ret.extend_from_slice(&self.class.to_be_bytes());
        ret.extend_from_slice(&self.ttl.to_be_bytes());
        ret.extend_from_slice(&self.rd_length.to_be_bytes());
        ret.extend_from_slice(&self.data.to_bytes());

        ret
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self, Self::Error> {
        let name = Name::from_bytes(bytes)?;
        let type_ = bytes.get_u16();
        let class = bytes.get_u16();
        let ttl = bytes.get_i32();
        let rd_length = bytes.get_u16();

        let mut data = Vec::new();
        std::io::Read::take(bytes, rd_length as u64)
            .read_to_end(&mut data)
            .or(Err(()))?;

        let data = RecordData::from_data(type_, &data)?;

        Ok(Self {
            name,
            type_,
            class,
            ttl,
            rd_length,
            data,
        })
    }
}
