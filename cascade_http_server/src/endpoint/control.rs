use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use petgraph::graph::NodeIndex;
use tokio::sync::{Mutex, MutexGuard};
use cascade_component::component::ComponentMetadata;
use cascade_core::graph::graph_controller::CascadeController;
use cascade_core::graph::graph_controller_error::{StartComponentError, StopComponentError};
use log::info;
use crate::endpoint::{EndpointError, EndpointResult, get_idx_query_parameter};


/// Start a component in the graph from an index query parameter
/// This will fail if either:
///     The query parameter is not present or NaN
///     The component to start does not exist in the graph
/// This will NOT fail if:
///     The component experiences a runtime error
pub async fn start_component(
    controller: Arc<Mutex<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let node_idx: NodeIndex = NodeIndex::new(get_idx_query_parameter(request)?);

    let mut controller_lock: MutexGuard<CascadeController> = controller.lock().await;

    // Try and start the component associated with the node
    let result: Result<ComponentMetadata, StartComponentError> = controller_lock.start_component(node_idx).await;

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
            "Encountered {:?} when starting {}",
            err,
            node_idx.index()
        ))),
    }
}

/// Stop a component in the graph from an index query parameter
/// This will fail if either:
///     The query parameter is not present or NaN
///     The component to stop is not running already
/// Unlike start this request will block until the component is stopped
pub async fn stop_component(
    controller: Arc<Mutex<CascadeController>>,
    request: Request<Body>,
) -> EndpointResult {
    let node_idx: NodeIndex = NodeIndex::new(get_idx_query_parameter(request)?);

    let mut controller_lock: MutexGuard<CascadeController> = controller.lock().await;

    let result: Result<ComponentMetadata, StopComponentError> = controller_lock
        .stop_component(node_idx)
        .await;

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
            "Encountered {:?} when stopping {}",
            err,
            node_idx.index()
        ))),
    }
}