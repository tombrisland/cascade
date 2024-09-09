use std::sync::Arc;

use hyper::{Body, Request};
use petgraph::graph::{EdgeIndex, NodeIndex};
use serde::Serialize;
use tokio::sync::{RwLock, RwLockReadGuard};

use cascade_core::controller::{CascadeController, ConnectionsMap};
use cascade_core::graph::CascadeGraph;
use crate::endpoint::{create_json_body, get_idx_query_parameter, EndpointError, EndpointResult};

#[derive(Serialize)]
struct ConnectionMetric {
    idx: usize,
    name: String,
    count: usize,
    capacity: usize,
}

/// Describe the connection state
pub async fn stat_connection(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let edge_idx: EdgeIndex = EdgeIndex::new(get_idx_query_parameter(request)?);

    let controller_lock: RwLockReadGuard<CascadeController> = controller.read().await;

    let connections_lock: RwLockReadGuard<ConnectionsMap> =
        controller_lock.connections.read().await;

    match connections_lock.get(&edge_idx) {
        None => Err(EndpointError::NotFound(format!(
            "No edge found at idx {}",
            edge_idx.index()
        ))),
        Some(connection) => Ok(create_json_body(&ConnectionMetric {
            idx: edge_idx.index(),
            name: connection.name.clone(),
            count: connection.tx.len(),
            capacity: connection.tx.capacity().unwrap(),
        })?),
    }
}

#[derive(Serialize)]
struct ComponentMetric {
    idx: usize,
    name: String,
    active_tasks: usize,
}

/// Describe the component state
pub async fn stat_component(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let node_idx: NodeIndex = NodeIndex::new(get_idx_query_parameter(request)?);

    let controller_lock: RwLockReadGuard<CascadeController> = controller.read().await;

    if let Some(execution) = controller_lock.executions.get(&node_idx) {

    }

    match controller_lock.executions.get(&node_idx) {
        None => Err(EndpointError::NotFound(format!(
            "No active execution found at idx {}",
            node_idx.index()
        ))),
        Some(execution) => Ok(create_json_body(&ComponentMetric {
            idx: node_idx.index(),
            name: execution.component.metadata.display_name.clone(),
            active_tasks: execution.active_tasks(),
        })?),
    }
}
