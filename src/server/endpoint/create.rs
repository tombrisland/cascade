use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use log::info;
use petgraph::graph::{EdgeIndex, NodeIndex};
use tokio::sync::{Mutex, MutexGuard};

use crate::component::definition::ComponentDefinition;
use crate::connection::definition::ConnectionDefinition;
use crate::graph::controller::CascadeController;
use crate::graph::graph::GraphInternal;
use crate::server::endpoint::{deserialise_body, EndpointResult};

/// Create a component in the graph from a JSON request
/// This will fail if either:
///     The JSON is malformed or doesn't match ComponentDefinition
///     The named component does not exist in the registry
pub async fn create_component(
    controller: Arc<Mutex<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let def: ComponentDefinition = deserialise_body(request).await?;
    let type_name: String = def.type_name.clone();

    let controller_lock: MutexGuard<CascadeController> = controller.lock().await;

    // Error early if the component type is not known
    if !controller_lock
        .component_registry
        .is_known_component(&type_name)
    {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(format!(
                "Type {} does not exist in registry",
                type_name
            )))?);
    }

    // Create a node for the component definition in the graph
    let node_idx: NodeIndex = controller_lock
        .graph_definition
        .lock()
        .await
        .graph_internal
        .add_node(def);

    let message: String = format!(
        "Successfully created instance of {} at idx {}",
        type_name,
        node_idx.index()
    );

    info!("{}", message);

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(message))?)
}

/// Create a connection in the graph from a JSON request
/// This will fail if either:
///     The JSON is malformed or doesn't match ConnectionDefinition
///     The nodes referenced by the connection don't exist
pub async fn create_connection(
    controller: Arc<Mutex<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let def: ConnectionDefinition = deserialise_body(request).await?;

    let from: NodeIndex = NodeIndex::new(def.from);
    let to: NodeIndex = NodeIndex::new(def.to);

    let controller_lock: MutexGuard<CascadeController> = controller.lock().await;
    let graph_internal: &mut GraphInternal =
        &mut controller_lock.graph_definition.lock().await.graph_internal;

    // Check whether the nodes from the definition exist in the graph

    // Add the edge between two defined nodes
    let index: EdgeIndex = graph_internal.add_edge(from, to, def);

    let message: String = format!(
        "Created connection idx {} between {} and {}",
        index.index(),
        from.index(),
        to.index()
    );

    info!("{}", message);

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(message))?)
}
