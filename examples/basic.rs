use airship::resource::Resource;
use airship::server;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let routes = vec![
        ("test </> place", Resource {}),
        ("test </> route </> ::name::", Resource {}),
    ];
    server::run::<Resource>(addr, &routes);
}
