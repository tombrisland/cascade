use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};
use nanoid::nanoid;

#[derive(Debug, Clone)]
pub struct FlowItem {
    pub id: String,
    // Time the item was created in epoch nanos
    pub created_time: u128,
    // Map of string properties
    pub properties: HashMap<String, String>,
    pub content: HashMap<String, Vec<u8>>,
}

impl FlowItem {
    pub fn new(properties: HashMap<String, String>) -> FlowItem {
        FlowItem {
            id: nanoid!(),
            created_time: SystemTime::now()
                .duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            // No content to begin with
            content: Default::default(),
            properties,
        }
    }
}