use std::sync::Arc;
use cascade_api::component::component::{ComponentMetadata, Schedule};
use cascade_api::component::Process;

pub struct Component {
    pub metadata: ComponentMetadata,
    pub schedule: Schedule,

    // Underlying producer to call
    pub implementation: Arc<dyn Process>,
}