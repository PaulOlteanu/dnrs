use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr};

use rand::seq::SliceRandom;
use rand::Rng;

use super::host::Host;

const ROOT_NAMESERVERS: [Ipv4Addr; 13] = [
    Ipv4Addr::new(198, 41, 0, 4),
    Ipv4Addr::new(199, 9, 14, 201),
    Ipv4Addr::new(192, 33, 4, 12),
    Ipv4Addr::new(199, 7, 91, 13),
    Ipv4Addr::new(192, 203, 230, 10),
    Ipv4Addr::new(192, 5, 5, 241),
    Ipv4Addr::new(192, 112, 36, 4),
    Ipv4Addr::new(198, 97, 190, 53),
    Ipv4Addr::new(192, 36, 148, 17),
    Ipv4Addr::new(192, 58, 128, 30),
    Ipv4Addr::new(193, 0, 14, 129),
    Ipv4Addr::new(199, 7, 83, 42),
    Ipv4Addr::new(202, 12, 24, 33),
];

#[derive(Clone, Default, Debug)]
pub struct NsQueue {
    queue: HashMap<usize, Vec<Host>>,
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
        let seed = ROOT_NAMESERVERS
            .iter()
            .map(|ns| ns.clone().into())
            .collect();
        queue.insert(0, seed);
        Self {
            queue,
            inserted_levels: HashSet::from([0]),
            max_level: 0,
        }
    }

    pub fn insert<T>(&mut self, host: T, level: usize)
    where
        T: Into<Host>,
    {
        self.max_level = self.max_level.max(level);
        self.inserted_levels.insert(level);

        self.queue
            .entry(level)
            .or_insert(Vec::new())
            .push(host.into())
    }

    pub fn insert_multiple(&mut self, hosts: &[Host], level: usize) {
        self.max_level = self.max_level.max(level);
        self.inserted_levels.insert(level);

        self.queue
            .entry(level)
            .or_insert(Vec::new())
            .extend_from_slice(hosts)
    }

    /// Gets one of the addresses (randomly selected) at the highest priority
    /// Note that this is not guaranteed to be the same one every time
    /// That means 2 ns_queue.peek() operations in a row maight not be equal
    pub fn peek(&self) -> &Host {
        // This should never panic because our push and pop functions will guarantee a non-empty vec with the key of self.max_level
        let addrs = self.queue.get(&self.max_level).unwrap();
        addrs.choose(&mut rand::thread_rng()).unwrap()
    }

    // TODO: This implementation needs to be much better
    pub fn pop(&mut self) -> Host {
        let addrs = self.queue.get_mut(&self.max_level).unwrap();
        let idx = rand::thread_rng().gen_range(0..addrs.len());
        addrs.swap_remove(idx)
    }
}
