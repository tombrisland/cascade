extern crate core;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use ::log::LevelFilter;
use tokio::sync::oneshot::{channel, Receiver, Sender};

use crate::graph::controller::CascadeController;
use crate::graph::graph_builder::CascadeGraphBuilder;
use crate::logger::SimpleLogger;
use crate::processor::log_message::LogMessage;
use crate::processor::update_properties::UpdateProperties;
use crate::producer::generate_item::GenerateItem;

mod component;
mod graph;
mod connection;
mod logger;
mod processor;
mod producer;

static LOGGER: SimpleLogger = SimpleLogger;

#[tokio::main()]
async fn main() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Logger failed to initialise");

    let generate_item = GenerateItem {
        batch_size: 5,
        content: Option::from("con".to_string()),
    };

    // let get_file = GetFile { directory: Box::from(Path::new("C:\\Users\\tombr\\Documents\\Code\\cascade\\src\\test")) };

    let log_message = LogMessage {
        item_count: AtomicUsize::new(1),
        log_every_x: 1,
    };

    let mut graph_builder = CascadeGraphBuilder::new().add_producer(generate_item);

    for _ in 0..1 {
        let update_properties = UpdateProperties {
            updates: HashMap::from([("item".to_string(), "value".to_string())]),
        };

        graph_builder.connect_to_previous(update_properties);
    }

    graph_builder.connect_to_previous(log_message);

    let graph = graph_builder.build();

    let mut controller = CascadeController {
        graph: Arc::new(graph),
        producer_handles: vec![],
    };

    controller.start().await;

    let (_tx, rx): (Sender<bool>, Receiver<bool>) = channel();

    rx.await.unwrap();
}
