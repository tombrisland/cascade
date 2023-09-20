use std::net::SocketAddr;
use std::sync::Arc;

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use log::info;
use tokio::sync::Mutex;

use crate::graph::controller::CascadeController;
use crate::server::endpoint::{EndpointError, EndpointResult};
use crate::server::endpoint::control::{start_component, stop_component};
use crate::server::endpoint::create::{create_component, create_connection};

mod endpoint;

static NOT_FOUND: &[u8] = b"Resource not found";

pub struct CascadeServer {
    pub addr: SocketAddr,

    pub controller: Arc<Mutex<CascadeController>>,
}

async fn router(
    controller: Arc<Mutex<CascadeController>>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let controller: Arc<Mutex<CascadeController>> = Arc::clone(&controller);

    info!("Received request for {}", req.uri());

    let result: EndpointResult = match (req.method(), req.uri().path()) {
        (&Method::PUT, "/create_component") => create_component(controller, req).await,
        (&Method::PUT, "/create_connection") => create_connection(controller, req).await,
        (&Method::GET, "/start_component") => start_component(controller, req).await,
        (&Method::GET, "/stop_component") => stop_component(controller, req).await,
        // Return 404 not found response.
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(NOT_FOUND.into())
            .unwrap()),
    };

    // Handle any generic errors which fell through
    match result {
        Ok(response) => Ok(response),
        Err(err) => match err {
            EndpointError::ServerError(err) => Err(err),
            EndpointError::BadRequest(err) => Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(err.to_string()))
                .unwrap())
        },
    }
}

type ServerError = Box<dyn std::error::Error + Send + Sync>;

impl CascadeServer {
    pub async fn start(self) -> Result<(), hyper::Error> {
        info!("Starting server on {}", &self.addr);

        let service = make_service_fn(move |_| {
            let controller: Arc<Mutex<CascadeController>> = Arc::clone(&self.controller);

            async { Ok::<_, ServerError>(service_fn(move |req| router(controller.clone(), req))) }
        });

        let server = Server::bind(&self.addr).serve(service);

        info!("Server started on {}", &self.addr);

        server.await
    }
}
