use std::sync::Arc;

use hyper::{Body, Request};
use tokio::sync::{Mutex, MutexGuard};

use crate::component::definition::ComponentDefinition;
use crate::connection::definition::ConnectionDefinition;
use crate::graph::controller::CascadeController;
use crate::graph::graph::CascadeGraph;
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

/// List the component definitions in the graph
pub async fn list_graph_nodes(
    controller: Arc<Mutex<CascadeController>>,
    _: Request<Body>,
) -> EndpointResult {
    // TODO this could probably be a RwLock instead
    let controller_lock: MutexGuard<CascadeController> = controller.lock().await;
    let graph_lock: MutexGuard<CascadeGraph> = controller_lock.graph_definition.lock().await;

    let definitions: Vec<ComponentDefinition> = graph_lock
        .graph_internal
        .raw_nodes()
        .iter()
        .map(|node| node.weight.clone())
        .collect();

    Ok(create_json_body(&definitions)?)
}

/// List the connection definitions in the graph
pub async fn list_graph_connections(
    controller: Arc<Mutex<CascadeController>>,
    _: Request<Body>,
) -> EndpointResult {
    let controller_lock: MutexGuard<CascadeController> = controller.lock().await;
    let graph_lock: MutexGuard<CascadeGraph> = controller_lock.graph_definition.lock().await;

    let definitions: Vec<ConnectionDefinition> = graph_lock
        .graph_internal
        .raw_edges()
        .iter()
        .map(|node| node.weight.clone())
        .collect();

    Ok(create_json_body(&definitions)?)
}
