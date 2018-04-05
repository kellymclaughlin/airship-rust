extern crate hyper;
extern crate futures;

// use resource::{Resource, Webmachine};

use futures::future::Future;

use hyper::server::{Http, Request, Response, Service};
use hyper::{ Method, StatusCode};

struct Airship {
    route_spec: String
}

impl Service for Airship {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
         match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                Box::new(futures::future::ok(
                    Response::new().with_body("Try POSTing data to /echo")
                ))
            },
             (&Method::Post, "/echo") => {
                Box::new(futures::future::ok(
                    Response::new().with_body("POST data received")
                ))
            },
            _ => {
                Box::new(futures::future::ok(
                    Response::new().with_status(StatusCode::NotFound)
                ))
            }
        }
    }
}

pub fn run() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    // let airship = Airship {
    //     route_spec: String::from("haute/route")
    // };
    let server = Http::new().bind(&addr, || Ok(Airship { route_spec: String::from("haute/route")})).unwrap();
    server.run().unwrap();
}
