use std::{sync::{Arc, Mutex}, io, env};
use hyper::{Response, Body, Method, Request, Uri, Client};
use hyper_tls::HttpsConnector;
use crate::store::Store;

pub async fn get_stream(ctx: Arc<Mutex<Store>>) -> io::Result<Response<Body>> { 
    let should_refresh = ctx.lock().unwrap().should_refresh();
    if should_refresh {
        let can_refresh = ctx.lock().unwrap().can_refresh();
        if !can_refresh {
            // TODO: Respond with an error message
            unimplemented!()
        }
        // TODO: handle refreshing the token data
        // unimplemented!()
    }
    // TODO: get the user data from the twitch api
    let uri = "https://api.twitch.tv/helix/streams?user_login=codico".parse::<Uri>().unwrap();


    let access_token = ctx.lock().unwrap().access_token();

    let https = HttpsConnector::new();
    let client = Client::builder()
        .build::<_, hyper::Body>(https);

    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Client-Id", env::var("TWITCH_CLIENT_ID").unwrap())
        .header("content-type", "application/json")
        .body(Body::empty()).unwrap();

    let resp = client.request(req).await.unwrap();
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let data =  String::from_utf8(body_bytes.to_vec()).unwrap();
    // let should_refresh = ctx.lock().unwrap().should_refresh();
    // let can_refresh = ctx.lock().unwrap().can_refresh();
    let resp = Response::new(Body::from(data));
    Ok(resp) 
}
