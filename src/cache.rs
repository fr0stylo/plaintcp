use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::proto::RequestCommand;

pub mod middlewares;

pub trait CacheServer {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8>;
}

#[derive(Debug, Clone)]
pub struct Cache {
    storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl CacheServer for &Cache {
    fn on_request(&self, c: &RequestCommand) -> Vec<u8> {
        match c {
            RequestCommand::Get(key) => self.get(key).unwrap(),
            RequestCommand::Set(key, val) => self.set(key, val.clone()).unwrap(),
            RequestCommand::Delete(key) => self.delete(key).unwrap(),
            RequestCommand::Keys(take, skip) => self.keys(take.clone(), skip.clone()).unwrap(),
            _ => Vec::new(),
        }
    }
}

impl Cache {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        match self.storage.lock().unwrap().get(key) {
            None => Some(Vec::new()),
            Some(x) => Some(x.clone()),
        }
    }

    pub fn set(&self, key: &str, val: Vec<u8>) -> Option<Vec<u8>> {
        match self.storage.lock().unwrap().insert(key.to_owned(), val) {
            None => Some(Vec::new()),
            Some(x) => Some(x.clone()),
        }
    }

    pub fn delete(&self, key: &str) -> Option<Vec<u8>> {
        match self.storage.lock().unwrap().remove(key) {
            None => Some(Vec::new()),
            Some(x) => Some(x.clone()),
        }
    }

    pub fn keys(&self, take: usize, skip: usize) -> Option<Vec<u8>> {
        let res: Vec<String> = self
            .storage
            .lock()
            .unwrap()
            .keys()
            .map(|x| x.clone())
            .take(take)
            .skip(skip)
            .collect();
        Some(res.join("\r\n").into_bytes())
    }
}
