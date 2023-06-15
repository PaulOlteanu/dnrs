use std::collections::HashMap;

use crate::{Name, ResourceRecord};

pub type Cache = HashMap<Name, Vec<ResourceRecord>>;
