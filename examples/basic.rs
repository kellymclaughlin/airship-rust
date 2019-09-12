use hyper::{Body, Method, Request};
use mime;
use mime::Mime;

use webmachine_derive::*;
use airship::resource::{Resource, Webmachine};
use airship::server;
use airship::types::RequestState;

#[derive(Clone)]
struct GetResource;

impl Webmachine for GetResource {
    fn allowed_methods(&self) -> Vec<Method> {
        vec![Method::Get]
    }

    fn content_types_provided(&self) -> Vec<(Mime, fn(&Request) -> Body)> {
        vec![
            (mime::TEXT_PLAIN, |_x:&Request| Body::from("ok")),
            (mime::APPLICATION_JSON, |_x:&Request| Body::from("{\"key\": \"value\"}"))
        ]
    }
}

#[derive(Clone, Webmachine)]
enum MyResources {
    Get(GetResource),
    Res(Resource),

}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let routes = vec![
        ("test </> place", MyResources::Get(GetResource {})),
        ("test </> route </> ::name::", MyResources::Res(Resource {})),
    ];
    server::run(addr, &routes, &RequestState::new);
}
