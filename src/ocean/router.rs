use crate::controller::topic;
use crate::controller::Controller;
use crate::db;
use crate::json_rpc::request;
use crate::json_rpc::response;
use hyper::body;
use hyper::body::Buf;
use hyper::{Body, Method, Request, Response, StatusCode};
use serde_json;

pub async fn route(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    if req.method() != Method::POST || req.uri().path() != "/dive" {
        println!(
            "Bad request: method: {}, URL: {}",
            req.method().as_str(),
            req.uri().path()
        );
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Bad request"))
            .unwrap());
    }

    let whole_body = body::aggregate(req).await?;
    let bytes = whole_body.bytes();
    let raw_req = String::from_utf8(bytes.to_vec()).unwrap();

    println!("Request: {}", raw_req);

    let json_rpc_req: request::Request = serde_json::from_slice(bytes).unwrap();
    let json_rpc_resp = exec(json_rpc_req);
    let raw_resp = serde_json::to_string(&json_rpc_resp).unwrap();

    println!("Response: {}", raw_resp);

    Ok(Response::new(Body::from(raw_resp)))
}

fn exec(req: request::Request) -> response::Response {
    let method: Vec<&str> = req.method.split('.').collect();
    let name = method[0];
    let method = method[1];

    let db = db::Db::new();

    let controller = factory(name).unwrap();
    let result = controller.exec(&db, method, req.params);
    let response = response::Response {
        id: req.id.unwrap(),
        method: req.method,
        result: result,
        error: None,
    };

    response
}

fn factory(name: &str) -> Option<Box<impl Controller>> {
    match name {
        "topic" => Some(Box::new(topic::Topic::new())),
        _ => None,
    }
}
