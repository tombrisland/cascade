// use hyper::header::HeaderName;
// use hyper::{header, Body, Response, StatusCode};
// use petgraph::graph::NodeIndex;
// use petgraph::Graph;
// use std::sync::Arc;
//
// use crate::component::ComponentMetadata;
// use crate::flow::connection::Connection;
// use crate::flow::controller::FlowController;
// use crate::model::component::{ComponentRequest, ComponentType};
// use crate::model::connection::ConnectionRequest;
// use crate::processor::Processor;
// use crate::producer::Producer;
// use crate::server::util::response;
// use crate::server::ServerState;
// use crate::{Process, Produce};
//
// pub async fn list_nodes(state: Arc<ServerState>) -> Response<Body> {
//     let guard = state.graph_definition.lock().unwrap();
//
//     let nodes: Vec<&ComponentRequest> = guard.raw_nodes().iter().map(|node| &node.weight).collect();
//
//     let nodes_json = serde_json::to_string_pretty(&nodes).unwrap();
//
//     Response::builder()
//         .status(StatusCode::OK)
//         .header(header::CONTENT_TYPE, "application/json")
//         .body(Body::from(nodes_json))
//         .unwrap()
// }
//
// pub async fn list_connections(state: Arc<ServerState>) -> Response<Body> {
//     let guard = state.graph_definition.lock().unwrap();
//
//     let nodes: Vec<&ConnectionRequest> =
//         guard.raw_edges().iter().map(|edge| &edge.weight).collect();
//
//     let nodes_json = serde_json::to_string_pretty(&nodes).unwrap();
//
//     Response::builder()
//         .status(StatusCode::OK)
//         .header(header::CONTENT_TYPE, "application/json")
//         .body(Body::from(nodes_json))
//         .unwrap()
// }
