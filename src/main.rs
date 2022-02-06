use std::{net::SocketAddr, env};
use hyper::{service::{make_service_fn, service_fn}, Server};

use dotenv::dotenv;

use crate::store::Store;

mod handlers;
pub mod store;
pub mod router;

fn read_port() -> u16 {
    match env::var("PORT") {
        Ok(port_str)=> { 
            match port_str.parse() {
                Ok(port) => port,
                Err(_) => 8000,
            }
        },
        Err(_) => 8000,
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let addr = SocketAddr::from(([127, 0, 0, 1], read_port()));
    let service = make_service_fn(|_| async { 
        let store = Store::make_store();
        Ok::<_, hyper::Error>(service_fn(move |req| router::router(store.clone(), req)))
    });
    let server = Server::bind(&addr).serve(service);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
