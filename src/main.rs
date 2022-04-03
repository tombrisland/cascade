extern crate core;

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

use ::log::LevelFilter;

use crate::flow::builder::FlowBuilder;
use crate::flow::controller::FlowController;
use crate::flow::FlowGraph;
use crate::logger::SimpleLogger;
use crate::processor::log_message::LogMessage;
use crate::processor::update_properties::UpdateProperties;
use crate::producer::generate_item::GenerateItem;

mod flow;
mod processor;
mod producer;
mod logger;
mod component;

static LOGGER: SimpleLogger = SimpleLogger;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Logger failed to initialise");

    let generate_item = GenerateItem { batch_size: 1, content: Option::from("con".to_string()) };

    // let get_file = GetFile { directory: Box::from(Path::new("C:\\Users\\tombr\\Documents\\Code\\cascade\\src\\test")) };

    let update_properties = UpdateProperties {
        updates: HashMap::from([("item".to_string(), "value".to_string())]),
    };

    let log_message = LogMessage {
        item_count: AtomicUsize::new(1),
        log_every_x: 1_000,
    };

    let flow = FlowBuilder::new()
        .add_producer(generate_item)
        .connect_to_previous(update_properties)
        .connect_to_previous(log_message).build();

    let controller = FlowController { flow };

    controller.start_flow().await;
}
