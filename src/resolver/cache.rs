use std::collections::{HashMap, HashSet};

use dnrs::{Name, RecordType, ResourceRecord};

pub struct Cache(HashMap<Name, HashSet<ResourceRecord>>);

impl Cache {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert_record(&mut self, name: Name, record: ResourceRecord) {
        self.0
            .entry(name)
            .or_insert_with(HashSet::new)
            .replace(record);
    }

    pub fn insert_records(&mut self, name: Name, records: Vec<ResourceRecord>) {
        if !(self.0.contains_key(&name)) {
            self.0.insert(name.clone(), HashSet::new());
        }

        for record in records {
            self.0.get_mut(&name).unwrap().replace(record);
        }
    }

    pub fn get_record_set(&self, name: &Name) -> Option<&HashSet<ResourceRecord>> {
        self.0.get(name)
    }

    pub fn get_records_by_type(
        &mut self,
        name: &Name,
        t: RecordType,
    ) -> impl Iterator<Item = ResourceRecord> + '_ {
        self.0
            .entry(name.to_owned())
            .or_insert_with(HashSet::new)
            .iter()
            .filter(move |rr| rr.type_ == t)
            .cloned()
    }
}
