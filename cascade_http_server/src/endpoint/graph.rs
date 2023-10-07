use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use log::info;
use petgraph::graph::{EdgeIndex, NodeIndex};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use cascade_api::component::definition::ComponentDefinition;
use cascade_api::connection::definition::ConnectionDefinition;
use cascade_core::controller::CascadeController;
use cascade_core::graph::{CascadeGraph, GraphInternal};

use crate::endpoint::{
    create_json_body, deserialise_body, EndpointError, EndpointResult, get_idx_query_parameter,
};

/// List the component definitions in the graph
pub async fn list_graph_nodes(
    controller: Arc<RwLock<CascadeController>>,
    _: Request<Body>,
) -> EndpointResult {
    let controller_lock: RwLockReadGuard<CascadeController> = controller.read().await;
    let graph_lock: RwLockReadGuard<CascadeGraph> = controller_lock.graph_definition.read().await;

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
    controller: Arc<RwLock<CascadeController>>,
    _: Request<Body>,
) -> EndpointResult {
    let controller_lock: RwLockReadGuard<CascadeController> = controller.read().await;
    let graph_lock: RwLockReadGuard<CascadeGraph> = controller_lock.graph_definition.read().await;

    let definitions: Vec<ConnectionDefinition> = graph_lock
        .graph_internal
        .raw_edges()
        .iter()
        .map(|node| node.weight.clone())
        .collect();

    Ok(create_json_body(&definitions)?)
}

/// Create a component in the graph from a JSON request
/// This will fail if either:
///     The JSON is malformed or doesn't match ComponentDefinition
///     The named component does not exist in the registry
pub async fn create_component(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let def: ComponentDefinition = deserialise_body(request).await?;
    let type_name: String = def.type_name.clone();

    let controller_lock: RwLockWriteGuard<CascadeController> = controller.write().await;

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
        .write()
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
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let def: ConnectionDefinition = deserialise_body(request).await?;

    let from: NodeIndex = NodeIndex::new(def.source);
    let to: NodeIndex = NodeIndex::new(def.target);

    let controller_lock: RwLockWriteGuard<CascadeController> = controller.write().await;
    let graph_internal: &mut GraphInternal = &mut controller_lock
        .graph_definition
        .write()
        .await
        .graph_internal;

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

/// Remove a component from the graph from a JSON request
/// This will fail if either:
///     The component doesn't exist
///     There are connections attached to the node
pub async fn remove_component(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let node_idx: NodeIndex = NodeIndex::new(get_idx_query_parameter(request)?);

    let controller_lock: RwLockWriteGuard<CascadeController> = controller.write().await;
    let mut graph_lock: RwLockWriteGuard<CascadeGraph> =
        controller_lock.graph_definition.write().await;

    // Error if component is still running
    if controller_lock.executions.contains_key(&node_idx) {
        return Err(EndpointError::BadRequest(format!(
            "Component at idx {} is still running",
            node_idx.index()
        )));
    }

    match graph_lock.graph_internal.remove_node(node_idx) {
        None => Err(EndpointError::BadRequest(format!(
            "No node at idx {}",
            node_idx.index()
        ))),
        Some(_) => {
            let message: String = format!("Removed node at idx {}", node_idx.index());

            info!("{}", message);

            Ok(Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(Body::from(message))?)
        }
    }
}

/// Remove a connection in the graph from a JSON request
/// This will fail if either:
///     The connection does not exist
///     There are components attached to it still running
pub async fn remove_connection(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let edge_idx: EdgeIndex = EdgeIndex::new(get_idx_query_parameter(request)?);

    let controller_lock: RwLockWriteGuard<CascadeController> = controller.write().await;
    let mut graph_lock: RwLockWriteGuard<CascadeGraph> =
        controller_lock.graph_definition.write().await;

    let (node_source, node_dest): (NodeIndex, NodeIndex) = graph_lock
        .graph_internal
        .edge_endpoints(edge_idx)
        .ok_or(EndpointError::BadRequest(format!(
            "No connection at idx {}",
            edge_idx.index()
        )))?;

    if controller_lock.executions.contains_key(&node_source)
        || controller_lock.executions.contains_key(&node_dest)
    {
        return Err(EndpointError::BadRequest(
            "Connected node is still running".to_string(),
        ));
    }

    // We just asserted that the edge exists
    graph_lock.graph_internal.remove_edge(edge_idx).unwrap();

    let message: String = format!("Removed connection at idx {}", edge_idx.index());

    info!("{}", message);

    Ok(Response::builder()
        .status(StatusCode::ACCEPTED)
        .body(Body::from(message))?)
}
