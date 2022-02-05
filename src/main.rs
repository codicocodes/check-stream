use std::net::SocketAddr;
use hyper::{service::{make_service_fn, service_fn}, Server};

use dotenv::dotenv;

use crate::store::Store;

mod handlers;
pub mod store;
pub mod router;



#[tokio::main]
async fn main() {
    dotenv().ok();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let service = make_service_fn(|_| async { 
        let store = Store::make_store();
        Ok::<_, hyper::Error>(service_fn(move |req| router::router(store.clone(), req)))
    });

    let server = Server::bind(&addr).serve(service);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}


