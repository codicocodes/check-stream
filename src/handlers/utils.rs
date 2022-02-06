use std::env;

pub fn get_check_twitch_id() -> String {
    match env::var("CHECK_TWITCH_ID") {
         Ok (twitch_id) => twitch_id,
         Err(_) => "41701337".to_string()
    }
}

