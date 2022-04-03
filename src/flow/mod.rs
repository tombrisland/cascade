use std::sync::Arc;

use petgraph::Graph;
use petgraph::graph::NodeIndex;

use crate::flow::builder::FlowComponent;
use crate::flow::connection::FlowConnection;

pub mod controller;
pub mod item;
pub mod connection;
pub mod builder;
mod configuration;

// Represents a graph of the entire flow
pub struct FlowGraph {
    pub graph: Arc<Graph<FlowComponent, FlowConnection>>,

    producer_indices: Vec<NodeIndex>,
}