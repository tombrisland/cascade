use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use cascade_api::component::{NamedComponent, Process};
use cascade_api::component::environment::ExecutionEnvironment;
use cascade_api::component::error::ComponentError;
use cascade_api::message::Message;

#[derive(Serialize, Deserialize)]
pub struct UpdateProperties {
    // Key value pairs to update on item properties
    pub updates: HashMap<String, String>,
}

impl NamedComponent for UpdateProperties {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "UpdateProperties"
    }
}

#[async_trait]
impl Process for UpdateProperties {
    fn create_from_json(config: Value) -> Arc<dyn Process> {
        let update_properties: UpdateProperties = serde_json::from_value(config).unwrap();

        Arc::new(update_properties)
    }

    async fn process(&self, execution: &mut ExecutionEnvironment) -> Result<(), ComponentError> {
        let mut item: Message = execution.recv().await?.clone();

        // Loop through updates and update FlowItem
        for (key, value) in self.updates.clone() {
            item.properties.insert(key, value);
        }

        execution.send_default(item).await
    }
}
