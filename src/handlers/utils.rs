use std::env;

use hyper::body::Bytes;
use serde::{Serialize, Deserialize};

pub fn get_check_twitch_id() -> String {
    match env::var("CHECK_TWITCH_ID") {
         Ok (twitch_id) => twitch_id,
         Err(_) => "41701337".to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    id: String,
    login: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stream {
    id: String,
    user_id: String,
    user_login: String,
    user_name: String,
    title: String,
    game_name: String,
    viewer_count: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct TwitchResponse<T> {
    data: Vec<T>
}

impl User {
    pub fn id(&self) -> String {
        return self.id.clone()
    }
}

pub fn bytes_to_stream(bytes: Bytes) -> Option<Stream> {
    let response_result: Result<TwitchResponse<Stream>, serde_json::Error> = serde_json::from_slice(&bytes);
    let mut response = match response_result {
         Ok(response)=>  response,
         Err(e)=> {
            eprintln!("{}", e);
            return  None
         },
    };
    response.data.pop()
}

pub fn bytes_to_user(bytes: Bytes) -> Option<User> {
    let response_result: Result<TwitchResponse<User>, serde_json::Error> = serde_json::from_slice(&bytes);
    let mut response = match response_result {
         Ok(response)=>  response,
         Err(e)=> {
            eprintln!("{}", e);
            return  None
         },
    };
    response.data.pop()
}

