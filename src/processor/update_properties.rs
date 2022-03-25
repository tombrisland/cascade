use std::collections::HashMap;

use crate::component::{Control, Controllable};
use crate::flow::item::FlowItem;
use crate::processor::{Changes, Process, ProcessError};

pub struct UpdateProperties {
    pub updates: HashMap<String, String>,
    pub control: Control,
}

impl Controllable for UpdateProperties {
    fn control(&self) -> &Control { &self.control }
}

impl Process for UpdateProperties {
    fn process(&self, mut item: FlowItem) -> Result<FlowItem, ProcessError> {
        self.updates.clone().into_iter().for_each(|(key, value)| {
            item.properties.insert(key, value);
        });

        Result::Ok(item)
    }

    fn will_edit(&self) -> Changes {
        Changes {
            properties: self.updates.keys().cloned().collect(),
            content: false,
        }
    }
}