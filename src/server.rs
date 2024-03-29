use std::net::SocketAddr;
use std::sync::Arc;

use futures::Future;
use hyper::server::{Http, Request, Response, Service};
use hyper::StatusCode;

use crate::decision;
use crate::resource::Webmachine;
use crate::route;
use crate::route::{RoutingSpec, RoutingTrie};
use crate::types::HasAirshipState;

struct Airship<R, S, F>
where
    S: HasAirshipState,
    R: Webmachine + Clone,
    F: Fn() -> S,
{
    routes: Arc<RoutingTrie<R>>,
    new_request_state: F,
}

impl<R, S, F> Airship<R, S, F>
where
    S: HasAirshipState,
    R: Webmachine + Clone,
    F: Fn() -> S,
{
    fn new(
        routes: Arc<RoutingTrie<R>>,
        new_request_state: F,
    ) -> Airship<R, S, F> {
        Airship {
            routes: Arc::clone(&routes),
            new_request_state,
        }
    }
}

impl<R, S, F> Service for Airship<R, S, F>
where
    S: HasAirshipState,
    R: Webmachine + Clone,
    F: Fn() -> S,
{
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<dyn Future<Item = Response, Error = hyper::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match route::route(&(*self.routes), req.path().to_string()) {
            Some(routed_resource) => {
                let r = &(routed_resource.0).1;
                let mut request_state = (self.new_request_state)();
                decision::traverse::<R, S>(&r, &req, &mut request_state)
            }
            None => Box::new(futures::future::ok(
                Response::new().with_status(StatusCode::NotFound),
            )),
        }
    }
}

pub fn run<R: 'static, S>(
    addr: SocketAddr,
    routes: &[(&str, R)],
    state_fun: &'static dyn Fn() -> S,
) where
    S: HasAirshipState,
    R: Webmachine + Clone,
{
    let routing_spec = RoutingSpec(routes.to_owned());
    let routing_trie = Arc::new(RoutingTrie::from(routing_spec));
    let server = Http::new()
        .bind(&addr, move || {
            Ok(Airship::new(Arc::clone(&routing_trie), state_fun))
        })
        .unwrap();
    server.run().unwrap();
}
