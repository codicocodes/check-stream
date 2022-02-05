use std::{collections::HashMap, sync::{Mutex, Arc}, env};
use hyper::{Request, Body, Response, StatusCode, Client, Method};
use url::Url;
use hyper::Uri;
use hyper_tls::HttpsConnector;

use crate::store::Store;
use std::io;

/// JSON response Callback Succeeded 
const SUCCESS: &str = "{\"success\": true}";

/// JSON response Callback Failed 
const FAILED: &str = "{\"success\": false}";

// const CHECK_TWITCH_ID = "41701337"

// HACK: do not hardcode values

fn parse_query_params (req: Request<Body>) -> HashMap<String, String> {
    let mut full_url: String = env::var("BASE_URL").unwrap().to_owned();
    full_url.push_str(&req.uri().to_string());
    let url_result = full_url.parse::<Url>();
    let parsed_url = match url_result {
        Ok(url) => url,
        Err(error) => panic!("Problem parsing the url: {:?}", error) ,
    };
    let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
    return hash_query
}

fn get_token_url (code: &str) -> String {
    return format!(
        "https://id.twitch.tv/oauth2/token\
        ?client_id={}\
        &client_secret={}\
        &grant_type=authorization_code\
        &code={}\
        &redirect_uri={}/login/callback",
        env::var("TWITCH_CLIENT_ID").unwrap(),
        env::var("TWITCH_CLIENT_SECRET").unwrap(),
        code,
        env::var("BASE_URL").unwrap(),
    )
}

/// Callback route that redirects to twitch for oAuth2
pub async fn callback (ctx: Arc<Mutex<Store>>, req: Request<Body>) -> io::Result<Response<Body>> {
    let hash_query = parse_query_params(req);
    let code_param = hash_query.get("code");
    match code_param {
        Some(code) => {
            let https = HttpsConnector::new();
            let client = Client::builder()
                .build::<_, hyper::Body>(https);

            let url = &get_token_url(code).parse::<Uri>().unwrap();

            let req = Request::builder()
                .method(Method::POST)
                .uri(url)
                .header("content-type", "application/json")
                .body(Body::empty()).unwrap();

            let resp = client.request(req).await.unwrap();
            let bytes = hyper::body::to_bytes(resp.into_body()).await.unwrap();

            // TODO: check user id before saving into store

            match ctx.lock().unwrap().parse_bytes(bytes) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error parsing token_data from bytes: {}", e);
                    let mut bad_request = Response::new(Body::from(FAILED));
                    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(bad_request)
                },
            };

            println!("Poisoned mutex? {}", ctx.is_poisoned());

            let access_token = ctx.lock().unwrap().access_token();
            // let uri = "https://api.twitch.tv/helix/streams?login=codico".parse::<Uri>().unwrap();
            let uri = "https://api.twitch.tv/helix/users".parse::<Uri>().unwrap();

            let req = Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Client-Id", env::var("TWITCH_CLIENT_ID").unwrap())
                .header("content-type", "application/json")
                .body(Body::empty()).unwrap();

            println!("Bearer {}", access_token);
            let https = HttpsConnector::new();
            let client = Client::builder()
                .build::<_, hyper::Body>(https);
            let resp = client.request(req).await.unwrap();

            async fn body_to_string(req: Response<Body>) -> String {
                let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
                String::from_utf8(body_bytes.to_vec()).unwrap()
            }

            let body_str = body_to_string(resp).await;

            println!("{}", body_str);

            Ok(Response::new(Body::from(SUCCESS)))
        }
        None => { 
            eprintln!("Did not receive a code param.");
            let mut bad_request = Response::new(Body::from(FAILED));
            *bad_request.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(bad_request)
        }
    }
}
