// Required to call trait fns dynamically
#![feature(fn_traits)]
extern crate core;

use std::sync::Arc;

use log::LevelFilter;
use tokio::sync::Mutex;

use crate::component::{NamedComponent, Process};
use crate::component_registry::{ComponentMap, ComponentRegistry};
use crate::component_std::generate_item::GenerateItem;
use crate::component_std::log_message::LogMessage;
use crate::component_std::update_properties::UpdateProperties;
use crate::graph::controller::CascadeController;
use crate::logger::SimpleLogger;
use crate::server::{CascadeService, ServerState};

mod component;
mod connection;
mod graph;
mod logger;
mod server;
mod component_registry;
mod component_std;

static LOGGER: SimpleLogger = SimpleLogger;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), hyper::Error> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Logger failed to initialise");

    let mut components: ComponentMap = Default::default();

    components.insert(GenerateItem::type_name(), GenerateItem::create_from_json);
    components.insert(LogMessage::type_name(), LogMessage::create_from_json);
    components.insert(
        UpdateProperties::type_name(),
        UpdateProperties::create_from_json,
    );

    let state = ServerState {
        controller: Arc::new(Mutex::new(CascadeController::new(ComponentRegistry::new(
            components,
        )))),
    };

    let service = CascadeService {
        addr: "127.0.0.1:3000".parse().unwrap(),
        state: Arc::new(state),
    };

    service.start().await
}
