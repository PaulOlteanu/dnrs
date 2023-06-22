use std::net::Ipv4Addr;

use derivative::Derivative;
use dnrs_macro::resource_record;

use crate::ByteSer;
use crate::{Name, RecordType};
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[resource_record]
pub struct A {
    pub ip: Ipv4Addr,
}

impl A {
    fn data_to_bytes(&self) -> Bytes {
        todo!()
    }
}
