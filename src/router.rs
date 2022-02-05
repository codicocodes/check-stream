use hyper::{Request, Body, Method, Response, StatusCode};
use crate::{store::Store, handlers};
use std::{io, sync::{Mutex, Arc}};

/// Router that matches the request with a handler based on method and path
pub async fn router(store: Arc<Mutex<Store>>, req: Request<Body>) -> io::Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/login") => handlers::login::login(store).await,
        (&Method::GET, "/login/callback") =>  handlers::callback::callback(store, req).await,
        (&Method::GET, "/codico") =>  handlers::get_stream::get_stream(store).await,
        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
