use std::collections::HashMap;
use hyper::{Body, http, Request, Response};
use hyper::body::{Buf, Bytes};
use serde::de::DeserializeOwned;
use serde_json::Error;

pub(crate) mod create;
pub(crate) mod control;

pub enum EndpointError {
    ServerError(hyper::Error),
    BadRequest(String),
}

impl From<hyper::Error> for EndpointError {
    fn from(value: hyper::Error) -> Self {
        EndpointError::ServerError(value)
    }
}

impl From<http::Error> for EndpointError {
    fn from(value: http::Error) -> Self {
        EndpointError::BadRequest(value.to_string())
    }
}

impl From<Error> for EndpointError {
    fn from(value: Error) -> Self {
        EndpointError::BadRequest(value.to_string())
    }
}

pub type EndpointResult = Result<Response<Body>, EndpointError>;

pub async fn deserialise_body<T: DeserializeOwned>(
    request: Request<Body>,
) -> Result<T, EndpointError> {
    // This is actually Infallible
    let whole_body: Bytes = hyper::body::to_bytes(request).await.unwrap();

    Ok(serde_json::from_reader(whole_body.reader())?)
}

pub fn parse_query_params(request: Request<Body>) -> HashMap<String, String> {
    request
        .uri()
        .query()
        .map(|v| {
            // Use url lib to parse out query param
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        // Default to empty map instead of failure
        .unwrap_or_else(HashMap::new)
}
