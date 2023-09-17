// use hyper::{Body, Response, StatusCode};
// use petgraph::graph::NodeIndex;
// use petgraph::Graph;
// use serde::{Deserialize, Serialize};
// use std::sync::{Arc, Mutex};
//
// use crate::component::Component;
// use crate::flow::connection::Connection;
// use crate::flow::controller::FlowController;
// use crate::model::component::ComponentType;
// use crate::processor::Processor;
// use crate::producer::Producer;
// use crate::server::model::component::ComponentType;
// use crate::server::util::response;
// use crate::server::ServerState;
// use crate::{Process, Produce};
//
// pub async fn start_flow(state: Arc<ServerState>) -> Response<Body> {
//     let definition_lock = state.graph_definition.lock().unwrap();
//
//     let mut flow_graph: Graph<Component, Connection> = Default::default();
//     let mut producer_indices: Vec<NodeIndex> = vec![];
//
//     definition_lock
//         .raw_nodes()
//         .iter()
//         .enumerate()
//         .for_each(|(idx, node)| {
//             let create_component = node.weight.clone();
//
//             let component = match create_component.component_type {
//                 ComponentType::PRODUCER => {
//                     let implementation: Arc<dyn Produce> = state
//                         .component_registry
//                         .producers
//                         .get(&*create_component.instance_type)
//                         .unwrap()
//                         .call((create_component.component_config,));
//
//                     producer_indices.push(NodeIndex::new(idx));
//
//                     Component::Producer(Arc::new(Producer {
//                         id: idx.to_string(),
//                         name: create_component.name,
//                         implementation,
//                         stop: Default::default(),
//                         active_instances: Default::default(),
//                         config: Default::default(),
//                     }))
//                 }
//                 ComponentType::PROCESSOR => {
//                     let implementation: Arc<dyn Process> = state
//                         .component_registry
//                         .processors
//                         .get(&*create_component.instance_type)
//                         .unwrap()
//                         .call((create_component.component_config,));
//
//                     Component::Processor(Arc::new(Processor {
//                         id: idx.to_string(),
//                         name: create_component.name,
//                         implementation,
//                     }))
//                 }
//             };
//
//             flow_graph.add_node(component);
//         });
//
//     definition_lock.raw_edges().iter().for_each(|edge| {
//         let create_connection = edge.weight.clone();
//
//         let from: NodeIndex = NodeIndex::new(create_connection.from);
//         let to: NodeIndex = NodeIndex::new(create_connection.to);
//
//         flow_graph.add_edge(from, to, Connection {});
//     });
//
//     let controller = FlowController {
//         graph: Arc::new(flow_graph),
//         producer_indices,
//         producer_handles: Arc::new(Mutex::new(vec![])),
//     };
//
//     let start_flow = Arc::new(controller);
//     let store = Arc::clone(&start_flow);
//
//     tokio::spawn(async move {
//         start_flow.start_flow().await;
//     });
//
//     state
//         .controller
//         .lock()
//         .map(|mut ctrl| {
//             *ctrl = Some(store);
//         })
//         .unwrap();
//
//     response(StatusCode::OK, String::from("Started"))
// }
