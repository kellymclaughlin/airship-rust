use std::sync::Arc;

use airship::resource::Resource;
use airship::route::RoutingSpec;
use airship::server;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let routes = [
        ("test </> place", Resource {}),
        ("test </> route </> ::name::", Resource {}),
    ];
    let route_vec = routes.to_vec();
    let route_arc = Arc::new(route_vec);
    // let routing_spec = RoutingSpec(route_vec);
    server::run::<Resource>(addr, route_arc);
}
