use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr};

use rand::seq::SliceRandom;
use rand::Rng;

const ROOT_NAMESERVERS: [IpAddr; 13] = [
    IpAddr::V4(Ipv4Addr::new(198, 41, 0, 4)),
    IpAddr::V4(Ipv4Addr::new(199, 9, 14, 201)),
    IpAddr::V4(Ipv4Addr::new(192, 33, 4, 12)),
    IpAddr::V4(Ipv4Addr::new(199, 7, 91, 13)),
    IpAddr::V4(Ipv4Addr::new(192, 203, 230, 10)),
    IpAddr::V4(Ipv4Addr::new(192, 5, 5, 241)),
    IpAddr::V4(Ipv4Addr::new(192, 112, 36, 4)),
    IpAddr::V4(Ipv4Addr::new(198, 97, 190, 53)),
    IpAddr::V4(Ipv4Addr::new(192, 36, 148, 17)),
    IpAddr::V4(Ipv4Addr::new(192, 58, 128, 30)),
    IpAddr::V4(Ipv4Addr::new(193, 0, 14, 129)),
    IpAddr::V4(Ipv4Addr::new(199, 7, 83, 42)),
    IpAddr::V4(Ipv4Addr::new(202, 12, 24, 33)),
];

#[derive(Clone, Default, Debug)]
pub struct NsQueue {
    queue: HashMap<usize, Vec<IpAddr>>,
    inserted_levels: HashSet<usize>,
    max_level: usize,
}

impl NsQueue {
    pub fn new() -> Self {
        Self {
            queue: HashMap::new(),
            inserted_levels: HashSet::from([0]),
            max_level: 0,
        }
    }

    pub fn seeded() -> Self {
        let mut queue = HashMap::new();
        queue.insert(0, ROOT_NAMESERVERS.into());
        Self {
            queue,
            inserted_levels: HashSet::from([0]),
            max_level: 0,
        }
    }

    pub fn insert(&mut self, addr: IpAddr, level: usize) {
        self.max_level = self.max_level.max(level);
        self.inserted_levels.insert(level);

        self.queue.entry(level).or_insert(Vec::new()).push(addr)
    }

    pub fn insert_multiple(&mut self, addrs: &[IpAddr], level: usize) {
        self.max_level = self.max_level.max(level);
        self.inserted_levels.insert(level);

        self.queue
            .entry(level)
            .or_insert(Vec::new())
            .extend_from_slice(addrs)
    }

    /// Gets one of the addresses (randomly selected) at the highest priority
    /// Note that this is not guaranteed to be the same one every time
    /// That means 2 ns_queue.peek() operations in a row maight not be equal
    pub fn peek(&self) -> &IpAddr {
        // This should never panic because our push and pop functions will guarantee a non-empty vec with the key of self.max_level
        let addrs = self.queue.get(&self.max_level).unwrap();
        addrs.choose(&mut rand::thread_rng()).unwrap()
    }

    pub fn pop(&mut self) -> IpAddr {
        // This should never panic because our push and pop functions will guarantee a non-empty vec with the key of self.max_level
        let addrs = self.queue.get_mut(&self.max_level).unwrap();
        let idx = rand::thread_rng().gen_range(0..addrs.len());
        addrs.swap_remove(idx)
    }
}
