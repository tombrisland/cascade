use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use hyper::body::Buf;
use petgraph::graph::NodeIndex;
use serde_json::Error;
use tokio::sync::MutexGuard;

use crate::component::definition::ComponentDefinition;
use crate::graph::controller::CascadeController;
use crate::server::ServerState;
use crate::server::util::response;

pub async fn create_component(state: Arc<ServerState>, request: Request<Body>) -> Response<Body> {
    let whole_body = hyper::body::to_bytes(request).await.unwrap();

    let result: Result<ComponentDefinition, Error> = serde_json::from_reader(whole_body.reader());

    if result.is_err() {
        return response(StatusCode::BAD_REQUEST, result.err().unwrap().to_string());
    }

    let def: ComponentDefinition = result.unwrap();
    let type_name: String = def.type_name.clone();

    let controller_lock: MutexGuard<CascadeController> = state.controller.lock().await;

    if !controller_lock
        .component_registry
        .is_known_component(&type_name)
    {
        return response(
            StatusCode::BAD_REQUEST,
            format!("Component type {} not known", &type_name),
        );
    }

    let node_idx: NodeIndex = controller_lock
        .graph_definition
        .lock().await
        .graph_internal
        .add_node(def);

    response(
        StatusCode::CREATED,
        format!(
            "Created instance of {} at node index {}",
            type_name,
            node_idx.index()
        ),
    )
}
