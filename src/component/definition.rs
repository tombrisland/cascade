use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub enum ComponentType {
    Producer,
    Processor,
}

#[derive(Clone, Deserialize)]
pub struct ComponentDefinition {
    pub display_name: String,

    pub type_name: String,
    pub component_type: ComponentType,

    pub config: Value,
}
