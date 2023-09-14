use std::sync::Arc;

use petgraph::Graph;
use petgraph::graph::NodeIndex;

use crate::connection::ConnectionEdge;
use crate::flow::graph::{CascadeGraph, GraphInternal};
use crate::processor::{Process, Processor};
use crate::producer::{Produce, Producer, ProducerConfig};

// Represents a graph of the entire flow
pub struct CascadeGraphBuilder {
    pub graph_internal: GraphInternal,

    // Last node added
    last_index: Option<NodeIndex>,
}

// Represent a single component in the flow
pub enum ComponentNode {
    Producer(Arc<Producer>),
    Processor(Arc<Processor>),
}

impl CascadeGraphBuilder {
    pub fn new() -> CascadeGraphBuilder {
        CascadeGraphBuilder {
            graph_internal: Graph::new(),
            last_index: None,
        }
    }

    // Add producer with no outgoing connections
    pub fn add_producer<T: 'static + Produce>(mut self, producer: T) -> CascadeGraphBuilder {
        let index = self
            .graph_internal
            .add_node(ComponentNode::Producer(Arc::new(Producer::new(
                producer,
                ProducerConfig {
                    schedule_per_second: 1,
                    concurrency: 1,
                },
            ))));

        // Set the last_index to enable connect_to_previous
        self.last_index = Some(index);

        self
    }

    // Add a processor connected to the previously defined Producer / Processor
    pub fn connect_to_previous<T: 'static + Process>(&mut self, processor: T) -> &CascadeGraphBuilder {
        if self.last_index.is_some() {
            let destination =
                self.graph_internal
                    .add_node(ComponentNode::Processor(Arc::new(Processor::new(
                        processor,
                    ))));

            let edge_id: String = format!("{}-{}", self.last_index.unwrap().index(), destination.index());

            self.graph_internal
                .add_edge(self.last_index.unwrap(), destination, Arc::new(ConnectionEdge::new(edge_id, 10_000)));

            self.last_index = Some(destination);
        }

        self
    }

    pub fn build(self) -> CascadeGraph {
        CascadeGraph {
            graph_internal: self.graph_internal,
        }
    }
}
