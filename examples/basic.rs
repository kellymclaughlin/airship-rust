use std::time::{Duration, SystemTime};

use hyper::header::HttpDate;
use hyper::{Body, Method, Request};
use mime;
use mime::Mime;

use airship::resource::{Resource, Webmachine};
use airship::server;
use airship::types::{HasAirshipState, RequestState};
use webmachine_derive::*;

#[derive(Clone)]
struct GetResource;

impl Webmachine for GetResource {
    fn allowed_methods<S: HasAirshipState>(
        &self,
        _state: &mut S,
    ) -> Vec<Method> {
        vec![Method::Get]
    }

    fn content_types_provided<S: HasAirshipState>(
        &self,
        _state: &mut S,
    ) -> Vec<(Mime, fn(&Request) -> Body)> {
        vec![
            (mime::TEXT_PLAIN, |_x: &Request| Body::from("ok")),
            (mime::APPLICATION_JSON, |_x: &Request| {
                Body::from("{\"key\": \"value\"}")
            }),
        ]
    }

    fn last_modified<S: HasAirshipState>(
        &self,
        _state: &mut S,
    ) -> Option<HttpDate> {
        let modified = SystemTime::now() - Duration::from_secs(60 * 60 * 24);
        Some(modified.into())
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
    server::run::<MyResources, RequestState>(addr, &routes, &RequestState::new);
}
