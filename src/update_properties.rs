use std::collections::HashMap;

use crate::component::{Process, ProcessError};
use crate::Item;

pub struct UpdateProperties {
    pub updates: HashMap<String, String>,
}

impl Process for UpdateProperties {
    fn process(&self, mut item: Item) -> Result<Item, ProcessError> {
        self.updates.clone().into_iter().for_each(|(key, value)| {
            item.properties.insert(key, value);
        });

        Result::Ok(item)
    }
}