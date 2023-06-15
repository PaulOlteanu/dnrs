use std::collections::HashMap;

use dnrs::{Name, ResourceRecord};

pub type Cache = HashMap<Name, Vec<ResourceRecord>>;
