use std::sync::Arc;

use log::{debug, info, warn};
use petgraph::graph::NodeIndex;
use petgraph::{Direction, Graph};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time;

use crate::flow::connection::FlowConnection;
use crate::flow::item::FlowItem;
use crate::flow::FlowComponent;
use crate::processor::Process;
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
                let (tx, mut rx): (Sender<FlowItem>, Receiver<FlowItem>) = channel(10_000);
                let graph = Arc::clone(graph);
                let producer = Arc::clone(producer);

                tokio::spawn(async move {
                    while let Some(item) = rx.recv().await {
                        run_flow(&graph, idx, item).await;
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

        interval.tick().await;

        // Skip adding another instance if there are too many
        if instance.instances_maxed() {
            continue;
        }

        instance.add_instance();

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

            instance.remove_instance();
        });
    }
}

// Run the flow through recursively to the end
async fn run_flow(
    graph: &Arc<Graph<FlowComponent, FlowConnection>>,
    producer_index: NodeIndex,
    input: FlowItem,
) {
    let mut queue: Vec<(FlowItem, NodeIndex)> = vec![(input, producer_index)];

    // While there are executions left to schedule
    while !queue.is_empty() {
        if let Some((input, idx)) = queue.pop() {
            let component = graph.node_weight(idx).unwrap();

            let output: FlowItem = match component {
                // For a producer pass through the input
                FlowComponent::Producer(_) => input,

                // A processor can be scheduled to run and the output propagated
                FlowComponent::Processor(processor) => {
                    let processor = Arc::clone(&processor.implementation);

                    // Execute the processor's work in a task
                    tokio::spawn(async move { processor.try_process(input).await })
                        .await
                        .unwrap()
                        .unwrap()
                }
            };

            graph
                // Retrieve downstream connections
                .neighbors_directed(idx, Direction::Outgoing)
                // Add them to the executions list
                .for_each(|idx| queue.push((output.clone(), idx)));
        }
    }
}
