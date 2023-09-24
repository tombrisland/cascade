use std::sync::Arc;

use hyper::{Body, Request};
use petgraph::graph::EdgeIndex;
use serde::Serialize;
use tokio::sync::{Mutex, MutexGuard, RwLockReadGuard};

use cascade_core::controller::{CascadeController, ConnectionsMap};

use crate::endpoint::{create_json_body, EndpointError, EndpointResult, get_idx_query_parameter};

#[derive(Serialize)]
struct ConnectionMetric {
    name: String,
    count: usize,
    max_items: usize,
}

/// Describe the connection state
pub async fn stat_connection(
    controller: Arc<Mutex<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let edge_idx: EdgeIndex = EdgeIndex::new(get_idx_query_parameter(request)?);

    let controller_lock: MutexGuard<CascadeController> = controller.lock().await;

    let connections_lock: RwLockReadGuard<ConnectionsMap> =
        controller_lock.connections.read().await;

    match connections_lock.get(&edge_idx) {
        None => Err(EndpointError::BadRequest(format!(
            "No edge found at idx {}",
            edge_idx.index()
        ))),
        Some(connection) => Ok(create_json_body(&ConnectionMetric {
            name: connection.name.clone(),
            count: connection.tx.len(),
            max_items: connection.max_items,
        })?),
    }
}
