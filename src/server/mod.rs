use std::net::SocketAddr;
use std::sync::Arc;

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use log::info;
use tokio::sync::Mutex;

use crate::graph::controller::CascadeController;
use crate::server::endpoint::create_component::create_component;
use crate::server::endpoint::create_connection::create_connection;
use crate::server::endpoint::start_component::start_component;

pub mod endpoint;
mod util;

static NOT_FOUND: &[u8] = b"Resource not found";

type ServerError = Box<dyn std::error::Error + Send + Sync>;

pub struct ServerState {
    pub controller: Arc<Mutex<CascadeController>>,
}

pub struct CascadeService {
    pub addr: SocketAddr,

    pub state: Arc<ServerState>,
}

async fn router(
    state: Arc<ServerState>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let state = Arc::clone(&state);

    info!("Received request for {}", req.uri());

    let res = match (req.method(), req.uri().path()) {
        (&Method::PUT, "/create_component") => create_component(state, req).await,
        (&Method::PUT, "/create_connection") => create_connection(state, req).await,
        (&Method::GET, "/start_component") => start_component(state, req).await,
        // Return 404 not found response.
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(NOT_FOUND.into())
            .unwrap(),
    };

    Ok(res)
}

impl CascadeService {
    pub async fn start(self) -> Result<(), hyper::Error> {
        let service = make_service_fn(move |_| {
            let state = Arc::clone(&self.state);

            async { Ok::<_, ServerError>(service_fn(move |req| router(state.clone(), req))) }
        });

        let server = Server::bind(&self.addr).serve(service);

        info!("Server started on {}", &self.addr);

        server.await
    }
}
