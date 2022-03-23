use std::collections::HashMap;
use std::fmt::{Debug};
use std::time::{SystemTime, UNIX_EPOCH};

const CREATED_TIME: &str = "created_time";

#[derive(Debug, Clone)]
pub struct Item {
    // Map of string properties
    pub properties: HashMap<String, String>,
}

impl Item {
    pub fn new() -> Item {
        let now: u128 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let properties =
            HashMap::from([(CREATED_TIME.to_string(), now.to_string())]);

        return Item {
            properties,
        };
    }
}