use std::collections::HashMap;

use async_trait::async_trait;

use crate::component::{Component, ComponentError};
use crate::graph::item::CascadeItem;
use crate::processor::Process;

const NAME: &str = "UpdateProperties";

pub struct UpdateProperties {
    // Key value pairs to update on item properties
    pub updates: HashMap<String, String>,
}

impl Component for UpdateProperties {
    fn name(&self) -> &'static str {
        NAME
    }
}

#[async_trait]
impl Process for UpdateProperties {
    fn on_initialisation(&self) {}

    async fn try_process(&self, mut item: CascadeItem) -> Result<CascadeItem, ComponentError> {
        // Loop through updates and update FlowItem
        for (key, value) in self.updates.clone() {
            item.properties.insert(key, value);
        }

        Ok(item)
    }
}