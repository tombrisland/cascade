use std::sync::Arc;

use log::{debug, error, info, warn};
use petgraph::{Direction, Graph};
use petgraph::graph::NodeIndex;
use tokio::time;

use crate::flow::connection::FlowConnection;
use crate::flow::FlowComponent;
use crate::flow::item::FlowItem;
use crate::FlowGraph;
use crate::processor::Process;
use crate::producer::{Produce, Producer};

// Control the execution of a FlowGraph
pub struct FlowController {
    pub flow: FlowGraph,

    // Map of indices to active threads
    // pub handles: Map<NodeIndex, JoinHandle<()>>
}

impl FlowController {
    pub async fn start_flow(&self) {
        let graph = &self.flow.graph;

        // Get a list of producers
        let producers: &Vec<usize> = &self.flow.producer_indices;

        info!("Starting flow with {} entry point(s)", producers.len());

        for idx in producers {
            let node_idx = NodeIndex::new(idx.clone());

            let producer = graph.node_weight(node_idx).unwrap();

            if let FlowComponent::Producer(inst) = producer {
                // Start each producer loop
                self.start_producer(node_idx, inst.clone()).await;
            }
        }
    }

    pub async fn start_producer(&self, idx: NodeIndex, instance: Arc<Producer>) {
        // Create an interval to produce at the configured rate
        let mut interval = time::interval(instance.config().schedule_duration());

        let mut error_count: u8 = 0;

        loop {
            // New references to graph and producer instance
            let graph = Arc::clone(&self.flow.graph);
            let instance = Arc::clone(&instance);

            if instance.stop_requested() {
                info!("Producer {} was terminated", instance.id);

                break;
            }

            // Spawn a new task for each producer request
            tokio::spawn(async move {
                let producer = instance.producer();

                match producer.try_produce() {
                    Ok(result) => {
                        debug!("Producer {} created item id {}", instance.id, result.id);
                        // Perform downstream processing
                        recurse_flow(graph, idx, result);
                    }
                    Err(err) => {
                        // Disable the producer if it fails move than retry_count
                        if error_count >= producer.config().retry_count {
                            error!("Producer {} was terminated after failing {} times", instance.id, err);

                            return;
                        }

                        warn!("Producer {} failed to run with {}", instance.id, err);
                        error_count += 1;
                    }
                };
            });

            interval.tick().await;
        }
    }
}

fn recurse_flow(graph: Arc<Graph<FlowComponent, FlowConnection>>, idx: NodeIndex, input: FlowItem) {
    // Get the indices immediately downstream from this index
    let next_indices: Vec<NodeIndex> = graph.neighbors_directed(idx, Direction::Outgoing).collect();

    for next_idx in next_indices {
        // Index will always contain a component
        let component = graph.node_weight(next_idx).unwrap();

        if let FlowComponent::Processor(inst) = component {
            // New references to graph and processor
            let graph = Arc::clone(&graph);
            let proc = inst.processor();

            // Clone input for passing to processor
            let input = input.clone();

            // New task to process the item
            tokio::spawn(async move {
                match proc.process(input) {
                    Ok(output) => {
                        recurse_flow(graph, next_idx, output);
                    }
                    Err(err) => {
                        error!("An error occurred");
                    }
                }
            });
        }
    }
}