use std::sync::Arc;

use petgraph::Graph;

use crate::flow::builder::FlowComponent;
use crate::flow::connection::FlowConnection;
use crate::processor::Process;
use crate::producer::Produce;

pub mod controller;
pub mod item;
pub mod connection;
pub mod builder;

// Represents a graph of the entire flow
pub struct FlowGraph {
    pub graph: Arc<Graph<FlowComponent, FlowConnection>>,

    producer_indices: Vec<usize>,
}