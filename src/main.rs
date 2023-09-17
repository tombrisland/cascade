#![feature(fn_traits)]
extern crate core;

use std::collections::HashMap;
use std::sync::Arc;

use log::LevelFilter;
use tokio::sync::Mutex;

use crate::component::NamedComponent;
use crate::component_registry::registry::{ComponentMap, ComponentRegistry};
use crate::graph::controller::CascadeController;
use crate::logger::SimpleLogger;
use crate::processor::log_message::LogMessage;
use crate::processor::Process;
use crate::processor::update_properties::UpdateProperties;
use crate::producer::generate_item::GenerateItem;
use crate::producer::Produce;
use crate::server::{CascadeService, ServerState};

#[macro_use]
mod component;
mod connection;
mod graph;
mod logger;
mod server;
#[macro_use]
mod processor;
#[macro_use]
mod producer;
mod component_registry;

static LOGGER: SimpleLogger = SimpleLogger;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), hyper::Error> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Logger failed to initialise");

    let mut processors: ComponentMap<Arc<dyn Process>> = HashMap::new();
    processors.insert(LogMessage::type_name(), LogMessage::create);
    processors.insert(UpdateProperties::type_name(), UpdateProperties::create);

    let mut producers: ComponentMap<Arc<dyn Produce>> = HashMap::new();
    producers.insert(GenerateItem::type_name(), GenerateItem::create);

    let state = ServerState {
        controller: Arc::new(Mutex::new(CascadeController::new(ComponentRegistry::new(processors, producers))))
    };

    let service = CascadeService {
        addr: "127.0.0.1:3000".parse().unwrap(),
        state: Arc::new(state),
    };

    service.start().await
}
