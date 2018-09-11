extern crate hyper;
extern crate futures;

use resource::{Resource};
use decision;
use route;
use route::RoutingSpec;
use types::RequestState;

use futures::Future;
use hyper::server::{Http, Request, Response, Service};
use hyper::StatusCode;

struct Airship {
    route_spec: RoutingSpec<Resource>
}

impl Service for Airship {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // type Future = FutureResult<Response, hyper::Error>;
    type Future = Box<Future<Item=Response, Error=hyper::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        //TODO: match request against route_spec
        //TODO: If matched then run decision tree
        // match route::route(route::run_router(&self.route_spec), req.path().to_string()) {
        //     Some(routed_resource) => {
        //         let r = (routed_resource.0).1;
        //         decision::traverse::<Resource>(&r, &req, &mut RequestState::new())
        //     },
        //     None =>  {
        //         Box::new(futures::future::ok(
        //             Response::new().with_status(StatusCode::NotFound)
        //         ))
        //     }
        // }
        match req.path() {
            "/test/route" => {
                let r = Resource {};
                decision::traverse::<Resource>(&r, &req, &mut RequestState::new())
            },
            _ =>  {
                Box::new(futures::future::ok(
                    Response::new().with_status(StatusCode::NotFound)
                ))
            }
        }
    }
}

pub fn run() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Airship { route_spec: RoutingSpec(vec![(String::from("/test/route"), Resource {})]) })).unwrap();
    server.run().unwrap();
}
