use std::{sync::{Arc, Mutex}, io, env};
use hyper::{Response, Body, Method, Request, Uri, Client, body::Bytes, StatusCode};
use hyper_tls::HttpsConnector;
use crate::store::{Store, TokenData};
use super::utils::{get_check_twitch_id, bytes_to_stream};

fn get_refresh_url(refresh_token: String) -> String {
    return format!(
        "https://id.twitch.tv/oauth2/token\
        ?client_id={}\
        &client_secret={}\
        &grant_type=refresh_token\
        &refresh_token={}",
        env::var("TWITCH_CLIENT_ID").unwrap(),
        env::var("TWITCH_CLIENT_SECRET").unwrap(),
        refresh_token
    )
}

async fn refresh_token_data(refresh_token: String) -> Bytes {
    let uri = get_refresh_url(refresh_token);
    let https = HttpsConnector::new();
    let client = Client::builder()
        .build::<_, hyper::Body>(https);

    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .body(Body::empty()).unwrap();

    let resp = client.request(req).await.unwrap();
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let data =  String::from_utf8(body_bytes.to_vec()).unwrap();
    let resp = Response::new(Body::from(data));
    hyper::body::to_bytes(resp.into_body()).await.unwrap()
}

pub async fn get_stream(ctx: Arc<Mutex<Store>>) -> io::Result<Response<Body>> { 
    let should_refresh = ctx.lock().unwrap().should_refresh();
    if should_refresh {
        let can_refresh = ctx.lock().unwrap().can_refresh();
        if !can_refresh {
            eprintln!("Cannot refresh token because token data is missing.");
            let mut internal_server_error = Response::new(Body::from(""));
            *internal_server_error.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(internal_server_error)
        }
        println!("Refreshing token data because access_token has expired.");
        let refresh_token = ctx.lock().unwrap().refresh_token();
        let token_data_bytes: Bytes = refresh_token_data(refresh_token).await;
        let token_data_result: Result<TokenData, serde_json::Error> = serde_json::from_slice(&token_data_bytes);
        let token_data: TokenData  = match token_data_result {
            Ok(token_data)  => token_data,
            Err(e) => {
                eprintln!("Error parsing token_data from bytes: {}", e);
                let mut bad_request = Response::new(Body::from(""));
                *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(bad_request)
            },
        };
        match ctx.lock().unwrap().save_token_data(token_data){
            Ok(_)  => (),
            Err(e) => {
                eprintln!("Error saving refreshed token data: {}", e);
                let mut internal_server_error = Response::new(Body::from(""));
                *internal_server_error.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                return Ok(internal_server_error)
            },
        };
    }
    let uri = format!("https://api.twitch.tv/helix/streams?user_id={}", get_check_twitch_id() ).parse::<Uri>().unwrap();
    // let uri = format!("https://api.twitch.tv/helix/streams?user_login=dashducks").parse::<Uri>().unwrap();
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
    let stream_option = bytes_to_stream(body_bytes);
    let stream = match stream_option {
        Some(stream) => stream,
        None => {
            let resp = Response::new(Body::from("{\"isLive\": false}"));
            return Ok(resp) 
        }
    };
    let stream_json = serde_json::to_string(&stream)?;
    let resp = Response::new(Body::from(stream_json));
    Ok(resp) 
}
