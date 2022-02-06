use std::{collections::HashMap, sync::{Mutex, Arc}, env, fmt::Debug};
use hyper::{Request, Body, Response, StatusCode, Client, Method};
use serde::{Serialize, Deserialize};
use url::Url;
use hyper::Uri;
use hyper_tls::HttpsConnector;

use crate::store::{Store, TokenData};
use std::io;

use super::utils::get_check_twitch_id;

/// JSON response Callback Succeeded 
const SUCCESS: &str = "{\"success\": true}";

/// JSON response Callback Failed 
const FAILED: &str = "{\"success\": false}";

fn get_bad_request() -> Response<Body> {
    let mut bad_request = Response::new(Body::from(FAILED));
    *bad_request.status_mut() = StatusCode::BAD_REQUEST;
    return bad_request
}

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


#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: String,
    login: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserResponse {
    data: Vec<User>
}

async fn get_user(access_token: String) -> Option<User> {
    let uri = "https://api.twitch.tv/helix/users".parse::<Uri>().unwrap();
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Client-Id", env::var("TWITCH_CLIENT_ID").unwrap())
        .header("content-type", "application/json")
        .body(Body::empty()).unwrap();

    let https = HttpsConnector::new();
    let client = Client::builder()
        .build::<_, hyper::Body>(https);
    let resp = client.request(req).await.unwrap();
    let bytes = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let user_response_result: Result<UserResponse, serde_json::Error> = serde_json::from_slice(&bytes);

    let mut user_response = match user_response_result {
         Ok(user_response)=>  user_response,
         Err(_)=> {
            return  None
         },
    };

    return user_response.data.pop()
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
            let token_data_result: Result<TokenData, serde_json::Error> = serde_json::from_slice(&bytes);
            let token_data: TokenData  = match token_data_result {
                Ok(token_data)  => token_data,
                    Err(e) => {
                        eprintln!("Error parsing token_data from bytes: {}", e);
                        let mut bad_request = Response::new(Body::from(FAILED));
                        *bad_request.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(bad_request)
                    },
            };

            let maybe_user: Option<User> = get_user(token_data.access_token()).await;
            match maybe_user {
                Some(user) => {
                    if user.id != get_check_twitch_id() {
                        return Ok(get_bad_request())
                    }
                }
                None => return Ok(get_bad_request())
            }

            match ctx.lock().unwrap().save_token_data(token_data) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error saving token data: {}", e);
                    return Ok(get_bad_request())
                },
            };
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
