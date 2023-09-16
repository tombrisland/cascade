use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use petgraph::graph::NodeIndex;
use tokio::sync::MutexGuard;

use crate::graph::controller::CascadeController;
use crate::server::ServerState;
use crate::server::util::response;

pub async fn start_component(state: Arc<ServerState>, request: Request<Body>) -> Response<Body> {
    let params: HashMap<String, String> = request
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    let x = params.get("node_idx").unwrap();
    let idx = usize::from_str(x).unwrap();

    let mut controller_lock: MutexGuard<CascadeController> = state.controller.lock().await;
    
    controller_lock.start_component(NodeIndex::new(idx)).await.unwrap();

    response(
        StatusCode::OK,
        format!(
            "Started component at id {}",
            idx
        ),
    )
}
