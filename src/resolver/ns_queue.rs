use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use dnrs::Name;
use indexmap::{IndexMap, IndexSet};
use rand::Rng;

type QueueLevel = usize;
type HostLists = (IndexMap<Name, IpAddr>, IndexSet<Name>);
type Queue = HashMap<QueueLevel, HostLists>;

#[derive(Clone, Default, Debug)]
pub struct NsQueue {
    queue: Queue,
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

    pub fn insert(&mut self, name: Name, ip: Option<IpAddr>, level: usize) {
        self.max_level = self.max_level.max(level);
        self.inserted_levels.insert(level);

        let lists: &mut HostLists = self
            .queue
            .entry(level)
            .or_insert((IndexMap::new(), IndexSet::new()));
        if let Some(ip) = ip {
            lists.0.insert(name, ip);
        } else {
            lists.1.insert(name);
        }
    }

    /// Gets one of the addresses (randomly selected) at the highest priority
    /// Note that this is not guaranteed to be the same one every time
    /// That means 2 ns_queue.peek() operations in a row maight not be equal
    pub fn peek(&self) -> Option<(&Name, Option<IpAddr>)> {
        let lists = self.queue.get(&self.max_level)?;

        if !lists.0.is_empty() {
            let idx = rand::thread_rng().gen_range(0..lists.0.len());
            lists
                .0
                .get_index(idx)
                .map(|(name, addr)| (name, Some(*addr)))
        } else if !lists.1.is_empty() {
            let idx = rand::thread_rng().gen_range(0..lists.1.len());
            lists.1.get_index(idx).map(|name| (name, None))
        } else {
            None
        }
    }

    // TODO: This implementation needs to be much better
    pub fn pop(&mut self) -> Option<(Name, Option<IpAddr>)> {
        let lists = self.queue.get_mut(&self.max_level)?;
        if !lists.0.is_empty() {
            let idx = rand::thread_rng().gen_range(0..lists.0.len());
            lists
                .0
                .swap_remove_index(idx)
                .map(|(name, ip)| (name, Some(ip)))
        } else if !lists.1.is_empty() {
            let idx = rand::thread_rng().gen_range(0..lists.1.len());
            lists.1.swap_remove_index(idx).map(|name| (name, None))
        } else {
            None
        }
    }
}
