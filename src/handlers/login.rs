use std::{io, sync::{Mutex, Arc}};

use hyper::{Response, Body};

use crate::store::Store;

const TWITCH_CLIENT_ID: &str = "2dz4adkj1vomxtnhfk82nbzz4e3vnt";
const BASE_URL: &str = "http://localhost:8000";

fn get_login_url() -> String {
    format!("https://id.twitch.tv/oauth2/authorize\
        ?client_id={}\
        &redirect_uri={}/login/callback\
        &response_type=code\
        &scope=openid\
        &claims={{\"id_token\":{{\"email_verified\":null}}}}", 
        TWITCH_CLIENT_ID,
        BASE_URL
    )
}

pub async fn login (mut _ctx: Arc<Mutex<Store>>) -> io::Result<Response<Body>> { 
    let login_url = get_login_url();
    let redirect = Response::builder()
        .status(302)
        .header("Location", login_url)
        .body(Body::empty())
        .unwrap();
    Ok(redirect) 
}
