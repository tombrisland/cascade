use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};

use log::{debug, error, info, warn};
use petgraph::{Direction, Graph};
use petgraph::graph::NodeIndex;
use tokio::time;

use crate::flow::connection::FlowConnection;
use crate::flow::FlowComponent;
use crate::flow::item::FlowItem;
use crate::FlowGraph;
use crate::producer::Producer;

// Control the execution of a FlowGraph
pub struct FlowController {
    pub flow: FlowGraph,
}

impl FlowController {
    pub async fn start_flow(&self) {
        let graph = &self.flow.graph;

        // Get a list of producers
        let producers: &Vec<NodeIndex> = &self.flow.producer_indices;

        info!("Starting flow with {} entry point(s)", producers.len());

        for idx in producers {
            let node_idx = idx.clone();

            let flow_component = graph.node_weight(node_idx).unwrap();

            if let FlowComponent::Producer(producer) = flow_component {
                let (tx, rx): (Sender<FlowItem>, Receiver<FlowItem>) = mpsc::channel();
                let graph = Arc::clone(graph);

                tokio::task::spawn_blocking(move || {
                    loop {
                        let graph = Arc::clone(&graph);

                        match rx.recv() {
                            Ok(item) => {
                                recurse_flow(graph, node_idx, item);
                            }
                            Err(_) => {}
                        }
                    }
                });

                self.start_producer(producer, tx).await;
            }
        }
    }

    pub async fn start_producer(&self, instance: &Arc<Producer>, tx: Sender<FlowItem>) {
        // Create an interval to produce at the configured rate
        let mut interval = time::interval(instance.config.schedule_duration());

        loop {
            // New reference to producer instance
            let instance = Arc::clone(&instance);

            // Clone sender to be passed to tasks
            let tx = tx.clone();

            if instance.should_stop() {
                info!("Producer was terminated");

                break;
            }

            // Spawn a new task for each producer request
            tokio::spawn(async move {
                let producer = &instance.implementation;

                match producer.try_produce(tx).await {
                    // Producer returns count of items emitted to channel
                    Ok(result) => {
                        match result {
                            Some(count) => {
                                debug!("Producer {} execution emitted {} items", producer.id(), count);
                            }
                            None => {
                                debug!("Producer {} execution emitted 0 items", producer.id());
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Producer failed to run with {}", err);
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

    if next_indices.len() == 1 {} else {}

    for next_idx in next_indices {
        // Index will always contain a component
        let component = graph.node_weight(next_idx).unwrap();

        if let FlowComponent::Processor(inst) = component {
            // New references to graph and processor
            let graph = Arc::clone(&graph);
            let proc = Arc::clone(&inst.implementation);

            // Clone input for passing to processor
            let input = input.clone();

            // New task to process the item
            tokio::spawn(async move {
                match proc.try_process(input).await {
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