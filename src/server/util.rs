use hyper::{Body, Response, StatusCode};

pub fn response(status: StatusCode, str: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(str))
        .unwrap()
}
