use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use log::info;
use petgraph::graph::NodeIndex;
use tokio::sync::{Mutex, MutexGuard};
use crate::component::component::ComponentMetadata;

use crate::graph::controller::CascadeController;
use crate::graph::error::{StartComponentError, StopComponentError};
use crate::server::endpoint::{parse_query_params, EndpointError, EndpointResult};

const INDEX_PARAM: &str = "idx";

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
    let node_idx: NodeIndex = get_node_idx_parameter(request)?;

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
                .status(StatusCode::CREATED)
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
    let node_idx: NodeIndex = get_node_idx_parameter(request)?;

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
                .status(StatusCode::CREATED)
                .body(Body::from(message))?)
        }
        Err(err) => Err(EndpointError::BadRequest(format!(
            "Encountered {:?} when stopping {}",
            err,
            node_idx.index()
        ))),
    }
}

fn get_node_idx_parameter(request: Request<Body>) -> Result<NodeIndex, EndpointError> {
    let params: HashMap<String, String> = parse_query_params(request);

    // Error if the param is missing or not an integer
    let idx_str: &String = params
        .get(INDEX_PARAM)
        .ok_or(EndpointError::BadRequest(format!(
            "Query parameter {} was missing",
            INDEX_PARAM
        )))?;
    let node_idx: usize = usize::from_str(idx_str).map_err(|_| {
        EndpointError::BadRequest(format!("Query parameter {} was not a number", INDEX_PARAM))
    })?;

    Ok(NodeIndex::new(node_idx))
}
