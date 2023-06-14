use std::collections::HashMap;

use crate::{Name, Question, RecordType, ResourceRecord};

pub type Cache = HashMap<CacheKey, Vec<ResourceRecord>>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub class: u16,
    pub type_: RecordType,
    pub name: Name,
}

impl CacheKey {
    pub fn new(class: u16, type_: RecordType, name: Name) -> Self {
        Self { class, type_, name }
    }
}

impl From<ResourceRecord> for CacheKey {
    fn from(value: ResourceRecord) -> Self {
        Self {
            class: value.class,
            type_: value.type_,
            name: value.name,
        }
    }
}

impl From<&ResourceRecord> for CacheKey {
    fn from(value: &ResourceRecord) -> Self {
        Self {
            class: value.class,
            type_: value.type_,
            name: value.name.clone(),
        }
    }
}

impl From<Question> for CacheKey {
    fn from(value: Question) -> Self {
        Self {
            class: value.class,
            type_: value.type_,
            name: value.name,
        }
    }
}

impl From<&Question> for CacheKey {
    fn from(value: &Question) -> Self {
        Self {
            class: value.class,
            type_: value.type_,
            name: value.name.clone(),
        }
    }
}
