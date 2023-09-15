use std::collections::HashMap;

use async_trait::async_trait;

use crate::component::{ComponentError, NamedComponent};
use crate::graph::item::CascadeItem;
use crate::processor::Process;

pub struct UpdateProperties {
    // Key value pairs to update on item properties
    pub updates: HashMap<String, String>,
}

impl NamedComponent for UpdateProperties {
    fn type_name() -> &'static str where Self: Sized {
        "UpdateProperties"
    }
}

#[async_trait]
impl Process for UpdateProperties {
    async fn process(&self, mut item: CascadeItem) -> Result<CascadeItem, ComponentError> {
        // Loop through updates and update FlowItem
        for (key, value) in self.updates.clone() {
            item.properties.insert(key, value);
        }

        Ok(item)
    }
}