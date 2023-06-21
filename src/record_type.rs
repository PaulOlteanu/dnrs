#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

impl RecordType {
    pub fn to_int(&self) -> u16 {
        *self as u16
    }

    pub fn from_int(v: u16) -> Option<Self> {
        match v {
            1 => Some(Self::A),
            2 => Some(Self::Ns),
            3 => Some(Self::Md),
            4 => Some(Self::Mf),
            5 => Some(Self::Cname),
            6 => Some(Self::Soa),
            7 => Some(Self::Mb),
            8 => Some(Self::Mg),
            9 => Some(Self::Mr),
            10 => Some(Self::Null),
            11 => Some(Self::Wks),
            12 => Some(Self::Ptr),
            13 => Some(Self::Hinfo),
            14 => Some(Self::Minfo),
            15 => Some(Self::Mx),
            16 => Some(Self::Txt),
            28 => Some(Self::Aaaa),
            41 => Some(Self::Opt),
            _ => None,
        }
    }
}
