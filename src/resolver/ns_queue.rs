use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use dnrs::Name;
use rand::seq::SliceRandom;
use rand::Rng;

type QueueLevel = usize;
type HostLists = (Vec<(Name, IpAddr)>, Vec<Name>);
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

        let lists = self.queue.entry(level).or_insert((Vec::new(), Vec::new()));
        if let Some(ip) = ip {
            lists.0.push((name, ip));
        } else {
            lists.1.push(name);
        }
    }

    /// Gets one of the addresses (randomly selected) at the highest priority
    /// Note that this is not guaranteed to be the same one every time
    /// That means 2 ns_queue.peek() operations in a row maight not be equal
    pub fn peek(&self) -> Option<(&Name, Option<IpAddr>)> {
        let lists = self.queue.get(&self.max_level)?;

        let mut rng = rand::thread_rng();

        if let Some((name, ip)) = lists.0.choose(&mut rng) {
            Some((name, Some(*ip)))
        } else {
            Some((lists.1.choose(&mut rng)?, None))
        }
    }

    // TODO: This implementation needs to be much better
    pub fn pop(&mut self) -> Option<(Name, Option<IpAddr>)> {
        let lists = self.queue.get_mut(&self.max_level)?;
        if !lists.0.is_empty() {
            let idx = rand::thread_rng().gen_range(0..lists.0.len());
            let result = lists.0.swap_remove(idx);
            Some((result.0, Some(result.1)))
        } else if !lists.1.is_empty() {
            let idx = rand::thread_rng().gen_range(0..lists.1.len());
            Some((lists.1.swap_remove(idx), None))
        } else {
            None
        }
    }
}
