use std::sync::Arc;

use crossbeam_channel::{bounded, Receiver, Sender};
use log::{debug, error, info, warn};
use petgraph::graph::NodeIndex;
use petgraph::{Direction, Graph};
use tokio::task::JoinHandle;
use tokio::time;

use crate::flow::connection::FlowConnection;
use crate::flow::item::FlowItem;
use crate::flow::FlowComponent;
use crate::processor::{Process, Processor};
use crate::producer::Producer;
use crate::FlowGraph;

// Control the execution of a FlowGraph
pub struct FlowController {
    pub flow: FlowGraph,

    pub producer_handles: Vec<JoinHandle<()>>,
}

impl FlowController {
    pub async fn start_flow(&mut self) {
        let graph = &self.flow.graph;

        // Get a list of producers in the flow
        let producers: &Vec<NodeIndex> = &self.flow.producer_indices;

        info!("Starting flow with {} entry point(s)", producers.len());

        for idx in producers {
            let idx = idx.clone();
            let component = graph.node_weight(idx).unwrap();

            if let FlowComponent::Producer(producer) = component {
                let (tx, rx): (Sender<FlowItem>, Receiver<FlowItem>) = bounded(1_000);
                let graph = Arc::clone(graph);
                let producer = Arc::clone(producer);

                tokio::task::spawn_blocking(move || loop {
                    let graph = Arc::clone(&graph);

                    match rx.recv() {
                        Ok(item) => tokio::spawn(async move {
                            run_flow(&graph, idx, item).await;
                        }),
                        Err(_) => {}
                    }
                });

                let handle = tokio::spawn(async move {
                    start_producer(producer, tx).await;
                });

                self.producer_handles.push(handle);
            }
        }
    }
}

async fn start_producer(instance: Arc<Producer>, tx: Sender<FlowItem>) {
    // Create an interval to produce at the configured rate
    let mut interval = time::interval(instance.config.schedule_duration());

    loop {
        // New reference to producer instance
        let instance = Arc::clone(&instance);

        // Clone sender to be passed to tasks
        let tx = tx.clone();

        if instance.should_stop() {
            info!("Producer {} was terminated", instance.implementation.id());

            break;
        }

        // Spawn a new task for each time the producer is scheduled
        tokio::spawn(async move {
            let producer = &instance.implementation;

            match producer.try_produce(tx).await {
                // Producer returns count of items emitted to channel
                Ok(result) => match result {
                    Some(count) => {
                        debug!(
                            "Producer {} execution emitted {} items",
                            producer.id(),
                            count
                        );
                    }
                    None => {
                        debug!("Producer {} execution emitted 0 items", producer.id());
                    }
                },
                Err(err) => {
                    warn!("Producer failed to run with {}", err);
                }
            };
        });

        interval.tick().await;
    }
}

// Recurse the flow graph passing the output to the input of the next processor in the flow
fn process_flow(graph: Arc<Graph<FlowComponent, FlowConnection>>, idx: NodeIndex, input: FlowItem) {
    // Get the indices immediately downstream from this index
    let next_indices: Vec<NodeIndex> = graph.neighbors_directed(idx, Direction::Outgoing).collect();

    debug!(
        "Found {} nodes immediately downstream of node {}",
        next_indices.len(),
        idx.index()
    );

    for next_idx in next_indices {
        // Retrieve the current processor from the graph
        let maybe_processor = retrieve_processor(&graph, next_idx);

        if maybe_processor.is_none() {
            // Item at the graph node was not a processor
            warn!(
                "Expected processor at node {} but was something else",
                idx.index()
            );

            return;
        }

        // New references to graph and processor
        let graph = Arc::clone(&graph);
        let processor = Arc::clone(&maybe_processor.unwrap().implementation);

        // Clone input for passing to processor
        let input = input.clone();

        // New task to process the item
        tokio::spawn(async move {
            match processor.try_process(input).await {
                Ok(output) => {
                    process_flow(graph, next_idx, output);
                }
                Err(err) => {
                    error!("An error occurred");
                }
            }
        });
    }
}

async fn run_flow(
    graph: &Arc<Graph<FlowComponent, FlowConnection>>,
    producer_index: NodeIndex,
    input: FlowItem,
) {
    let mut stack: Vec<(FlowItem, NodeIndex)> = vec![(input, producer_index)];

    loop {
        match stack.pop() {
            Some((input, idx)) => {
                let component = graph.node_weight(idx).unwrap();

                let output: FlowItem = match component {
                    FlowComponent::Producer(_) => input,
                    FlowComponent::Processor(processor) => {
                        let processor = Arc::clone(&processor.implementation);
                        // Execute the processor's work in a task
                        let handle: JoinHandle<FlowItem> =
                            tokio::spawn(
                                async move { processor.try_process(input).await.unwrap() },
                            );

                        handle.await.unwrap()
                    }
                };

                graph
                    // Retrieve downstream connections
                    .neighbors_directed(idx, Direction::Outgoing)
                    .for_each(|idx| stack.push((output.clone(), idx)));
            }
            // If there are no more downstream components the flow is complete
            None => break,
        }
    }
    // Get downstream for current node
    // Loop through downstream
}

fn retrieve_downstream_processors(
    graph: Arc<Graph<FlowComponent, FlowConnection>>,
    idx: NodeIndex,
) -> Vec<(NodeIndex, Arc<Processor>)> {
    let next_indices: Vec<NodeIndex> = graph.neighbors_directed(idx, Direction::Outgoing).collect();

    next_indices
        .iter()
        .filter_map(|idx| {
            let idx = idx.clone();

            // Retrieve component from graph via node index
            let component = graph.node_weight(idx).unwrap();

            if let FlowComponent::Processor(processor) = component {
                // Return if the component is a processor
                return Option::Some((idx, processor.clone()));
            }

            return Option::None;
        })
        .collect()
}

// Return the processor at a specified NodeIndex if it exists
fn retrieve_processor(
    graph: &Arc<Graph<FlowComponent, FlowConnection>>,
    idx: NodeIndex,
) -> Option<&Arc<Processor>> {
    // Retrieve component from graph via node index
    let component = graph.node_weight(idx).unwrap();

    if let FlowComponent::Processor(processor) = component {
        // Return if the component is a processor
        return Option::Some(processor);
    }

    return Option::None;
}
