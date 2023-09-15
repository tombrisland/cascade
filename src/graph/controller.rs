use std::sync::Arc;

use log::{debug, info, warn};
use petgraph::graph::NodeIndex;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::MissedTickBehavior::Delay;

use crate::connection::{ComponentOutput, ConnectionEdge};
use crate::graph::graph::CascadeGraph;
use crate::graph::graph_builder::ComponentNode;
use crate::graph::item::CascadeItem;
use crate::processor::{Process, Processor};
use crate::producer::Producer;

// Control the execution of a FlowGraph
pub struct CascadeController {
    pub graph: Arc<CascadeGraph>,

    pub producer_handles: Vec<JoinHandle<()>>,
}

impl CascadeController {
    pub async fn start(&mut self) {
        let graph: &Arc<CascadeGraph> = &self.graph;

        info!("Starting cascade graph with {} nodes", graph.get_nodes().len());

        for node_idx in graph.get_nodes() {
            let flow_component = graph.get_node_component(node_idx).unwrap();

            let graph: Arc<CascadeGraph> = Arc::clone(&graph);

            match flow_component {
                ComponentNode::Producer(producer) => {
                    let producer = Arc::clone(producer);

                    let handle: JoinHandle<()> = tokio::spawn(async move {
                        let component_output: ComponentOutput = graph.get_output(node_idx);

                        start_producer(producer, component_output).await
                    });

                    self.producer_handles.push(handle);
                }
                ComponentNode::Processor(processor) => {
                    let processor = Arc::clone(processor);

                    start_processor(processor, node_idx, graph);
                }
            }
        }
    }
}

fn start_processor(instance: Arc<Processor>, node_idx: NodeIndex, graph: Arc<CascadeGraph>) {
    // New reference to processor instance
    let instance: Arc<Processor> = Arc::clone(&instance);

    let incoming_connections: Vec<Arc<ConnectionEdge>> = graph.get_incoming_connections(node_idx);

    // No point starting a task if this component has no input
    if incoming_connections.is_empty() {
        return;
    }

    for incoming in incoming_connections {
        let graph: Arc<CascadeGraph> = Arc::clone(&graph);
        let instance: Arc<Processor> = Arc::clone(&instance);
        let incoming: Arc<ConnectionEdge> = Arc::clone(&incoming);

        // Task to execute processor on each input item
        tokio::spawn(async move {
            let processor: &Arc<dyn Process> = &instance.implementation;

            let component_output: ComponentOutput = graph.get_output(node_idx);

            loop {
                if let Some(input) = incoming.recv().await {
                    let output: CascadeItem = processor.process(input).await.unwrap();

                    component_output.send(output).await.expect("Something went wrong");
                }
            }
        });
    }
}

async fn start_producer(instance: Arc<Producer>, outbound: ComponentOutput) {
    // Create an interval to produce at the configured rate
    let mut interval = time::interval(instance.config.schedule_duration());

    // Don't try and catch up with missed ticks
    interval.set_missed_tick_behavior(Delay);

    loop {
        // New reference to producer instance
        let instance = Arc::clone(&instance);

        // Clone sender to be passed to tasks
        let outbound = outbound.clone();

        if instance.should_stop() {
            info!("Producer {} was terminated", instance.component.id);

            break;
        }

        interval.tick().await;

        // Skip adding another instance if there are too many
        if instance.concurrency_maxed() {
            continue;
        }

        instance.increment_concurrency();

        // Spawn a new task for each time the producer is scheduled
        tokio::spawn(async move {
            let producer = &instance.implementation;

            match producer.produce(outbound).await {
                // Producer returns count of items emitted to channel
                Ok(count) => {
                    debug!(
                            "Producer {} execution emitted {} items",
                            instance.component.id,
                            count
                        );
                }
                Err(err) => {
                    warn!("Producer failed to run with {}", err);
                }
            };

            instance.decrement_concurrency();
        });
    }
}