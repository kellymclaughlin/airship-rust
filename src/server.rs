use std::net::SocketAddr;
use std::sync::Arc;

use futures::Future;
use hyper::server::{Http, Request, Response, Service};
use hyper::StatusCode;

use crate::resource::Webmachine;
use crate::decision;
use crate::route;
use crate::route::{RoutingSpec, RoutingTrie};
use crate::types::RequestState;

// #[derive(Clone)]
struct Airship<R>
where
    R: Webmachine + Clone
{
    routes: RoutingTrie<R>
}

impl<R> Airship<R>
where
    R: Webmachine + Clone
{
    fn new(routes: Arc<RoutingSpec<R>>) -> Airship<R> {
        let routes_clone = Arc::clone(&routes);
        Airship {
            routes: RoutingTrie::from(*routes_clone)
        }
    }
}

impl<R> Service for Airship<R>
where
    R: Webmachine + Clone
{
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Response, Error=hyper::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match route::route(&self.routes, req.path().to_string()) {
            Some(routed_resource) => {
                let r = &(routed_resource.0).1;
                decision::traverse::<R>(&r, &req, &mut RequestState::new())
            },
            None =>  {
                Box::new(futures::future::ok(
                    Response::new().with_status(StatusCode::NotFound)
                ))
            }
        }
    }
}

pub fn run<R>(addr: SocketAddr, routes: Arc<Vec<(&str, R)>>)
where
    R: Webmachine + Clone
{
    // let addr = "127.0.0.1:3000".parse().unwrap();
    // let routes = [
    //     ("test </> place", Resource {}),
    //     ("test </> route </> ::name::", Resource {}),
    // ];
    // let app = Airship {
    //     route_spec: RoutingSpec(routes)
    // };
    // let server = Http::new().bind(&addr, || Ok(Airship::new(RoutingSpec(vec![(String::from("/test/route"), Resource {})])))).unwrap();
    // let spec_clone = routing_spec.clone();
    let routing_spec = RoutingSpec(Arc::clone(routes));
    // let app = Airship::new(routing_spec);
    let server = Http::new()
        .bind(&addr, move || Ok(Airship::new(routing_spec)))
        // .bind(&addr, move || Ok(app))
        .unwrap();
    server.run().unwrap();
}
