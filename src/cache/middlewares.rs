use std::fmt::Debug;
use std::time::SystemTime;

use crate::cache::CacheServer;
use crate::proto::RequestCommand;

#[derive(Debug)]
pub struct Logger<S> where S: CacheServer {
    inner: S,
}

impl<S> Logger<S> where S: CacheServer {
    pub fn new(inner: S) -> Self {
        Logger {
            inner
        }
    }
}

impl<S> CacheServer for Logger<S> where S: CacheServer {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8> {
        println!("{:?}", f);
        self.inner.on_request(f)
    }
}

impl<S> CacheServer for &Logger<S> where S: CacheServer {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8> {
        let t = SystemTime::now();
        let res = self.inner.on_request(f);
        println!("[{:?}] {:?}", t.elapsed().unwrap(), f);
        res
    }
}
