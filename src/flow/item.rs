use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FlowItem {
    pub id: uuid::Uuid,
    pub created_time: u128,
    // Map of string properties
    pub properties: HashMap<String, String>,
    pub content: Option<Vec<u8>>,
}

impl FlowItem {
    pub fn new(properties: HashMap<String, String>) -> FlowItem {
        FlowItem {
            // TODO clone method should set parent_id and replace id
            id: Uuid::new_v4(),
            created_time: SystemTime::now()
                .duration_since(UNIX_EPOCH).unwrap().as_millis(),
            // No content to begin with
            content: None,
            properties,
        }
    }
}