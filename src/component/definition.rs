use serde::Deserialize;
use serde_json::Value;

use crate::component::component::Schedule;

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
    #[serde(default = "schedule_default")]
    pub schedule: Schedule,

    pub config: Value,
}

pub fn schedule_default() -> Schedule {
    Schedule::Interval{
        // Default to once every half second
        period_millis: 500
    }
}
