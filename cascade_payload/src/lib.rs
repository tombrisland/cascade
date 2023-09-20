use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};
use nanoid::nanoid;


#[derive(Debug, Clone)]
pub struct CascadeItem {
    pub id: String,
    pub created_nanos: u128,
    // Map of string properties
    pub properties: HashMap<String, String>,
    pub content: HashMap<String, Vec<u8>>,
}

impl CascadeItem {
    pub fn new(properties: HashMap<String, String>) -> CascadeItem {
        CascadeItem {
            id: nanoid!(),
            created_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            // No content to begin with
            content: Default::default(),
            properties,
        }
    }
}