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

struct Airship<R>
where
    R: Webmachine + Clone
{
    routes: Arc<RoutingTrie<R>>
}

impl<R> Airship<R>
where
    R: Webmachine + Clone
{
    fn new(routes: Arc<RoutingTrie<R>>) -> Airship<R> {
        Airship {
            routes: Arc::clone(&routes)
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
        match route::route(&(*self.routes), req.path().to_string()) {
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

pub fn run<R: 'static>(addr: SocketAddr, routes: &Vec<(&str, R)>)
where
    R: Webmachine + Clone
{
    let routing_spec = RoutingSpec(routes.clone());
    let routing_trie = Arc::new(RoutingTrie::from(routing_spec));
    let server = Http::new()
        .bind(&addr, move || Ok(Airship::new(Arc::clone(&routing_trie))))
        .unwrap();
    server.run().unwrap();
}
