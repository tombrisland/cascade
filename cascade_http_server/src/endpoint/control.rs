use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use log::info;
use petgraph::graph::NodeIndex;
use tokio::sync::{RwLock, RwLockWriteGuard};

use cascade_api::component::component::ComponentMetadata;
use cascade_core::controller::CascadeController;
use cascade_core::controller::error::{StartComponentError, StopComponentError};

use crate::endpoint::{EndpointError, EndpointResult, get_idx_query_parameter};

/// Start a component in the graph from an index query parameter
/// This will fail if either:
///     The query parameter is not present or NaN
///     The component to start does not exist in the graph
/// This will NOT fail if:
///     The component experiences a runtime error
pub async fn start_component(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let node_idx: NodeIndex = NodeIndex::new(get_idx_query_parameter(request)?);

    let mut controller_lock: RwLockWriteGuard<CascadeController> = controller.write().await;

    // Try and start the component associated with the node
    let result: Result<ComponentMetadata, StartComponentError> =
        controller_lock.start_component(node_idx).await;

    match result {
        Ok(metadata) => {
            let message: String = format!(
                "Successfully started {} at idx {}",
                metadata,
                node_idx.index()
            );

            info!("{}", message);

            Ok(Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(Body::from(message))?)
        }
        Err(err) => Err(EndpointError::BadRequest(format!(
            "Encountered {:?} when starting idx {}",
            err,
            node_idx.index()
        ))),
    }
}

/// Stop a component in the graph from an index query parameter
/// This will fail if either:
///     The query parameter is not present or NaN
///     The component to stop is not running already
pub async fn stop_component(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let node_idx: NodeIndex = NodeIndex::new(get_idx_query_parameter(request)?);

    let mut controller_lock: RwLockWriteGuard<CascadeController> = controller.write().await;

    let result: Result<ComponentMetadata, StopComponentError> =
        controller_lock.stop_component(node_idx).await;

    // When the output queue is full up the thread is blocked on send
    // This means it cannot cycle round to read the shutdown signal
    // Could change it to check the queue length before scheduling another invocation
    // Then also have something to increase the size of the queue on stoppage??

    match result {
        Ok(metadata) => {
            let message: String = format!(
                "Successfully stopped {} at idx {}",
                metadata,
                node_idx.index()
            );

            info!("{}", message);

            Ok(Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(Body::from(message))?)
        }
        Err(err) => Err(EndpointError::BadRequest(format!(
            "Encountered {:?} when stopping idx {}",
            err,
            node_idx.index()
        ))),
    }
}

/// Kill a component in the graph from an index query parameter
/// This will fail if either:
///     The query parameter is not present or NaN
///     The component to stop is not running already
/// This will not return until the components are killed
pub async fn kill_component(
    controller: Arc<RwLock<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let node_idx: NodeIndex = NodeIndex::new(get_idx_query_parameter(request)?);

    let mut controller_lock: RwLockWriteGuard<CascadeController> = controller.write().await;

    controller_lock.kill_component(node_idx).await;

    let message: String = format!("Successfully sent kill signal to idx {}", node_idx.index());

    info!("{}", message);

    Ok(Response::builder()
        .status(StatusCode::ACCEPTED)
        .body(Body::from(message))?)
}
