use std::{sync::{Arc, Mutex}, io};
use hyper::{Response, Body};
use crate::store::Store;

pub async fn get_stream(ctx: Arc<Mutex<Store>>) -> io::Result<Response<Body>> { 
    let should_refresh = ctx.lock().unwrap().should_refresh();
    if should_refresh {
        let can_refresh = ctx.lock().unwrap().can_refresh();
        if !can_refresh {
            // TODO: Respond with an error message
            // unimplemented!()
        }
        // TODO: handle refreshing the token data
        // unimplemented!()
    }
    // TODO: get the user data from the twitch api
    let should_refresh = ctx.lock().unwrap().should_refresh();
    let can_refresh = ctx.lock().unwrap().can_refresh();
    let resp = Response::new(Body::from(format!("{{\"should_refresh\": {}, \"can_refresh\": {}}}", should_refresh, can_refresh )));
    Ok(resp) 
}
