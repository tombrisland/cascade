use std::collections::HashMap;

use async_trait::async_trait;

use crate::component::{Component, ComponentError};
use crate::flow::item::FlowItem;
use crate::processor::{Changes, Process};

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

    async fn try_process(&self, mut item: FlowItem) -> Result<FlowItem, ComponentError> {
        // Loop through updates and update FlowItem
        for (key, value) in self.updates.clone() {
            item.properties.insert(key, value);
        }

        Result::Ok(item)
    }

    fn will_edit(&self) -> Changes {
        Changes {
            properties: self.updates.keys().cloned().collect(),
            content: false,
        }
    }
}