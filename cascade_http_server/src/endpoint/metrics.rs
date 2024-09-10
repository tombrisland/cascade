use std::sync::Arc;

use crate::endpoint::{create_json_body, get_idx_query_parameter, EndpointError, EndpointResult};
use cascade_api::component::component::ComponentMetadata;
use cascade_api::connection::ConnectionMetadata;
use cascade_core::controller::{CascadeController, ConnectionsMap};
use hyper::{Body, Request};
use petgraph::graph::{EdgeIndex, NodeIndex};
use serde::Serialize;
use tokio::sync::{RwLock, RwLockReadGuard};

#[derive(Serialize)]
struct ConnectionMetric {
    metadata: ConnectionMetadata,
    count: usize,
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
            metadata: connection.metadata.clone(),
            count: connection.tx.len(),
        })?),
    }
}

#[derive(Serialize)]
struct ComponentMetric {
    metadata: ComponentMetadata,
    active_tasks: usize,
}

/// Describe the component state
pub async fn stat_component(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let node_idx: NodeIndex = NodeIndex::new(get_idx_query_parameter(request)?);

    let controller_lock: RwLockReadGuard<CascadeController> = controller.read().await;

    match controller_lock.executions.get(&node_idx) {
        None => Err(EndpointError::NotFound(format!(
            "No active execution found at idx {}",
            node_idx.index()
        ))),
        Some(execution) => Ok(create_json_body(&ComponentMetric {
            metadata: execution.component.metadata.clone(),
            active_tasks: execution.active_tasks(),
        })?),
    }
}
