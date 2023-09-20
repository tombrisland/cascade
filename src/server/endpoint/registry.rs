use std::sync::Arc;

use hyper::{Body, Request};
use tokio::sync::{Mutex, MutexGuard};

use crate::graph::controller::CascadeController;
use crate::server::endpoint::{create_json_body, EndpointResult};

/// List available component types in the component registry
pub async fn list_available_components(
    controller: Arc<Mutex<CascadeController>>,
    _: Request<Body>,
) -> EndpointResult {
    let controller_lock: MutexGuard<CascadeController> = controller.lock().await;
    let component_types: Vec<&str> = controller_lock.component_registry.list_component_types();

    Ok(create_json_body(&component_types)?)
}
