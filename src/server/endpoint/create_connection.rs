use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use hyper::body::Buf;
use petgraph::graph::{EdgeIndex, NodeIndex};
use serde_json::Error;

use crate::connection::definition::ConnectionDefinition;
use crate::graph::graph::GraphInternal;
use crate::server::ServerState;
use crate::server::endpoint::http_util::response;

pub async fn create_connection(state: Arc<ServerState>, request: Request<Body>) -> Response<Body> {
    let whole_body = hyper::body::to_bytes(request).await.unwrap();

    let result: Result<ConnectionDefinition, Error> = serde_json::from_reader(whole_body.reader());

    if result.is_err() {
        return response(StatusCode::BAD_REQUEST, result.err().unwrap().to_string());
    }

    let def: ConnectionDefinition = result.unwrap();

    let from: NodeIndex = NodeIndex::new(def.from);
    let to: NodeIndex = NodeIndex::new(def.to);

    let controller_lock = state.controller.lock().await;
    let graph_internal: &mut GraphInternal = &mut controller_lock.graph_definition.lock().await.graph_internal;

    let index: EdgeIndex = graph_internal.add_edge(from, to, def);

    return response(
        StatusCode::CREATED,
        format!(
            "Created connection {} between {} and {}",
            index.index(),
            from.index(),
            to.index()
        ),
    );
}
