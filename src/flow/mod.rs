use std::sync::Arc;

use petgraph::{Direction, Graph};
use petgraph::graph::NodeIndex;

use crate::flow::connection::{ComponentConnections, FlowConnection};
use crate::processor::{Process, Processor};
use crate::producer::{Produce, Producer};

pub mod controller;
pub mod item;
pub mod connection;

// Represents a graph of the entire flow
pub struct FlowGraph {
    pub graph: Graph<FlowComponent, FlowConnection>,

    // Last node added
    last_index: Option<NodeIndex>,
}

// Represent a single component in the flow
pub enum FlowComponent {
    Producer(Producer),
    Processor(Processor),
}

impl FlowGraph {
    pub fn new() -> FlowGraph {
        FlowGraph {
            graph: Graph::new(),
            last_index: None,
        }
    }

    // Add producer with no outgoing connections
    pub fn add_producer(mut self, producer: Arc<dyn Produce>) -> FlowGraph {
        let index = self.graph.add_node(FlowComponent::Producer(Producer::new(producer)));

        // Set the last_index to enable connect_to_previous
        self.last_index = Some(index);

        self
    }

    // Add a processor connected to the previously defined Producer / Processor
    pub fn connect_to_previous(mut self, processor: Arc<dyn Process>) -> FlowGraph {
        if self.last_index.is_some() {
            let destination = self.graph.add_node(FlowComponent::Processor(Processor::new(processor)));

            self.graph.add_edge(self.last_index.unwrap(), destination, FlowConnection::new());

            self.last_index = Some(destination);
        }

        self
    }

    fn component_connections(&self, node_idx: NodeIndex) -> ComponentConnections {
        ComponentConnections {
            incoming: self.connections(node_idx, Direction::Incoming),
            outgoing: self.connections(node_idx, Direction::Outgoing),
        }
    }

    fn connections(&self, node_idx: NodeIndex, direction: Direction) -> Vec<FlowConnection> {
        self.graph.edges_directed(node_idx, direction)
            .map(|edge| { edge.weight().clone() }).collect()
    }
}