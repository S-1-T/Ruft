use serde::{Deserialize, Serialize};

#[derive(PartialEq, Deserialize, Serialize, Debug)]
pub struct Entry {
    pub index: usize,
    pub term: u32,
    pub command: String,
}