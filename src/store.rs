use std::{time::{SystemTime, UNIX_EPOCH}, fs::File, io::{self, BufReader}, sync::{Mutex, Arc}};

use hyper::body::Bytes;
use serde::{Serialize, Deserialize};

pub struct Store {
    token_data: TokenData,
    refreshed_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

impl Store {
    pub fn new() -> Store {
        Store{
            token_data: 
                TokenData {
                    access_token: String::new(),
                    refresh_token: String::new(),
                    expires_in: 0,
                },
            refreshed_at: 0,
        }
    }

    pub fn make_store() -> Arc<Mutex<Store>> {
        let store_result = Store::read_from_json();
        println!("Creating store....");
        match store_result {
            Ok(store) => Arc::new(Mutex::new(store)),
            Err(error) => {
                eprintln!("error reading token data from file: {}", error);
                println!("initializing empty store");
                Arc::new(Mutex::new(Store::new()))
            }
        }
    }

    pub fn access_token(&self) -> String {
        self.token_data.access_token.clone()
    }

    pub fn set_access_token(&mut self, access_token: String) -> &Store {
        self.token_data.access_token = access_token;
        return self
    }

    pub fn refresh_token(&self) -> String {
        self.token_data.refresh_token.clone()
    }

    pub fn set_refresh_token(&mut self, refresh_token: String) -> &Store {
        self.token_data.refresh_token = refresh_token;
        return self
    }

    pub fn should_refresh(&self) -> bool {
        if self.refreshed_at == 0 {
            return true
        }
        let now =  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let limit = self.refreshed_at + self.token_data.expires_in;
        return now >= limit
    }

    pub fn can_refresh(&self) -> bool {
        if self.token_data.refresh_token == "" {
            return false
        }
        return true
    }

    pub fn set_expires_in(&mut self, expires_in: u64) -> &Store {
        self.token_data.expires_in = expires_in;
        return self
    }

    pub fn set_refreshed_at(&mut self, refreshed_at: u64) -> &Store {
        self.refreshed_at = refreshed_at;
        return self
    }

    pub fn parse_bytes(&mut self, bytes: Bytes) -> io::Result<&Store> {
        let token_data_result = serde_json::from_slice::<TokenData>(&bytes);
        let token_data: TokenData = match token_data_result {
            Ok(token_data)  => token_data,
            Err(e) => return Err(std::io::Error::from(e)),
        };
        ::serde_json::to_writer(&File::create("token.json")?, &token_data)?;
        self.token_data = token_data;
        return Ok(self)
    }

    pub fn read_from_json() -> io::Result<Store> {
        let file = File::open("token.json")?;
        let reader = BufReader::new(file);
        let token_data: TokenData = ::serde_json::from_reader(reader)?;
        let store = Store{token_data, refreshed_at:0 };
        Ok(store)
    } 
} 

pub fn get_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add_to_store() {
        let mut store = Store::new();
        assert_eq!(store.access_token(), "");
        assert_eq!(store.refresh_token(), "");
        assert_eq!(store.should_refresh(), true);
        store.set_access_token(String::from("abc"));
        store.set_refresh_token(String::from("bcd"));
        assert_eq!(store.access_token(), String::from("abc"));
        assert_eq!(store.refresh_token(), String::from("bcd"));
    }

    #[test]
    fn expires_in_future() {
        let mut store = Store::new();
        let now = get_time();
        store.set_refreshed_at(now);
        store.set_expires_in(10);
        assert_eq!(store.should_refresh(), false);
    }

    #[test]
    fn expires_in_past() {
        let mut store = Store::new();
        let now = get_time();
        let past = now - 100;
        store.set_refreshed_at(past);
        store.set_expires_in(0);
        assert_eq!(store.should_refresh(), true);
    }

    #[test]
    fn can_be_mutated() {
        fn mutating(mut ctx: Store) {
            ctx.set_access_token("abc".to_string());
            assert_eq!(ctx.access_token(), "abc");
        }
        let store = Store::new();
        mutating(store)
    }
}
