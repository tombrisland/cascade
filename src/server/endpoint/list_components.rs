// use hyper::header::HeaderName;
// use hyper::{header, Body, Response, StatusCode};
// use log::info;
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
// pub async fn list_processors(state: Arc<ServerState>) -> Response<Body> {
//     let processors: Vec<&&str> = state.component_registry.processors.keys().collect();
//
//     let result = serde_json::to_string_pretty(&processors);
//
//     if result.is_err() {
//         return response(
//             StatusCode::INTERNAL_SERVER_ERROR,
//             result.err().unwrap().to_string(),
//         );
//     }
//
//     Response::builder()
//         .status(StatusCode::OK)
//         .header(header::CONTENT_TYPE, "application/json")
//         .body(Body::from(result.unwrap()))
//         .unwrap()
// }
