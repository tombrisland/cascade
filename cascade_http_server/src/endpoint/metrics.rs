use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use petgraph::graph::EdgeIndex;
use tokio::sync::{Mutex, MutexGuard, RwLockReadGuard};

use cascade_core::controller::{CascadeController, ConnectionsMap};

use crate::endpoint::{get_idx_query_parameter, EndpointError, EndpointResult};

/// List the number of message on the connection
pub async fn list_connection_size(
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
        Some(connection) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(connection.tx.len().to_string()))
            .unwrap()),
    }
}
