// use hyper::{Body, Response, StatusCode};
// use petgraph::graph::NodeIndex;
// use petgraph::Graph;
// use std::sync::Arc;
//
// use crate::component::Component;
// use crate::flow::connection::Connection;
// use crate::flow::controller::FlowController;
// use crate::model::component::ComponentType;
// use crate::processor::Processor;
// use crate::producer::Producer;
// use crate::server::util::response;
// use crate::server::ServerState;
// use crate::{Process, Produce};
//
// pub async fn stop_flow(state: Arc<ServerState>) -> Response<Body> {
//     state
//         .controller
//         .lock()
//         .map(|option| {
//             match option.as_ref() {
//                 Some(flow_controller) => {
//                     let x = &flow_controller.producer_handles;
//
//                     x.lock()
//                         .map(|mut handles| {
//                             handles.iter().for_each(|handle| handle.abort());
//
//                             handles.clear();
//                         })
//                         .unwrap();
//                 }
//                 None => {}
//             };
//         })
//         .unwrap();
//
//     response(StatusCode::OK, String::from("Stopped flow"))
// }
