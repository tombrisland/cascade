use std::sync::Arc;

use petgraph::Graph;
use petgraph::graph::NodeIndex;

use crate::flow::connection::FlowConnection;
use crate::FlowGraph;
use crate::processor::{Process, Processor};
use crate::producer::{Produce, Producer, ProducerConfig};

// Represents a graph of the entire flow
pub struct FlowBuilder {
    pub graph: Graph<FlowComponent, FlowConnection>,

    producer_indices: Vec<NodeIndex>,

    // Last node added
    last_index: Option<NodeIndex>,
}

// Represent a single component in the flow
pub enum FlowComponent {
    Producer(Arc<Producer>),
    Processor(Arc<Processor>),
}

impl FlowBuilder {
    pub fn new() -> FlowBuilder {
        FlowBuilder {
            graph: Graph::new(),
            producer_indices: vec![],
            last_index: None,
        }
    }

    // Add producer with no outgoing connections
    pub fn add_producer<T: 'static + Produce>(mut self, producer: T) -> FlowBuilder {
        let index = self.graph.add_node(FlowComponent::Producer(Arc::new(Producer::new(producer, ProducerConfig { schedule_per_second: 35_000 }))));

        // Set the last_index to enable connect_to_previous
        self.last_index = Some(index);
        self.producer_indices.push(index);

        self
    }

    // Add a processor connected to the previously defined Producer / Processor
    pub fn connect_to_previous<T: 'static + Process>(mut self, processor: T) -> FlowBuilder {
        if self.last_index.is_some() {
            let destination = self.graph.add_node(FlowComponent::Processor(Arc::new(Processor::new(processor))));

            self.graph.add_edge(self.last_index.unwrap(), destination, FlowConnection::new());

            self.last_index = Some(destination);
        }

        self
    }

    pub fn build(self) -> FlowGraph {
        FlowGraph {
            graph: Arc::new(self.graph),
            producer_indices: self.producer_indices,
        }
    }
}