use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use dnrs::Name;

#[derive(Clone, Default, Debug)]
pub struct Host {
    name: Option<Name>,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
}

impl Host {
    pub fn new(name: Option<Name>, ipv4: Option<Ipv4Addr>, ipv6: Option<Ipv6Addr>) -> Option<Self> {
        if name.is_none() && ipv4.is_none() && ipv6.is_none() {
            None
        } else {
            Some(Self { name, ipv4, ipv6 })
        }
    }

    pub fn name(&self) -> &Option<Name> {
        &self.name
    }

    pub fn resolved(&self) -> bool {
        self.ipv4.is_some() || self.ipv6.is_some()
    }

    pub fn get_ip(&self) -> Option<IpAddr> {
        if let Some(ipv4) = self.ipv4 {
            Some(IpAddr::V4(ipv4))
        } else if let Some(ipv6) = self.ipv6 {
            Some(IpAddr::V6(ipv6))
        } else {
            None
        }
    }
}

impl From<Name> for Host {
    fn from(value: Name) -> Self {
        Self {
            name: Some(value),
            ipv4: None,
            ipv6: None,
        }
    }
}

impl From<Ipv4Addr> for Host {
    fn from(value: Ipv4Addr) -> Self {
        Self {
            name: None,
            ipv4: Some(value),
            ipv6: None,
        }
    }
}

impl From<Ipv6Addr> for Host {
    fn from(value: Ipv6Addr) -> Self {
        Self {
            name: None,
            ipv4: None,
            ipv6: Some(value),
        }
    }
}

impl From<(Name, Option<Ipv4Addr>, Option<Ipv6Addr>)> for Host {
    fn from(value: (Name, Option<Ipv4Addr>, Option<Ipv6Addr>)) -> Self {
        Self {
            name: Some(value.0),
            ipv4: value.1,
            ipv6: value.2,
        }
    }
}

impl From<(Option<Name>, Ipv4Addr, Option<Ipv6Addr>)> for Host {
    fn from(value: (Option<Name>, Ipv4Addr, Option<Ipv6Addr>)) -> Self {
        Self {
            name: value.0,
            ipv4: Some(value.1),
            ipv6: value.2,
        }
    }
}

impl From<(Option<Name>, Option<Ipv4Addr>, Ipv6Addr)> for Host {
    fn from(value: (Option<Name>, Option<Ipv4Addr>, Ipv6Addr)) -> Self {
        Self {
            name: value.0,
            ipv4: value.1,
            ipv6: Some(value.2),
        }
    }
}
