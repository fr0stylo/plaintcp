use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::mpsc::{channel, Sender};
use std::thread;
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

pub struct WriteLog<S: CacheServer> {
    inner: S,
    tx: Sender<RequestCommand>,
}

impl<S: CacheServer> CacheServer for &WriteLog<S> {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8> {
        self.tx.send(f.clone()).expect("[WAL] Failed to send message for sink");

        self.inner.on_request(f)
    }
}

impl<S: CacheServer> WriteLog<S> {
    pub fn new(path: &str, inner: S) -> Self {
        let (tx, rx) = channel::<RequestCommand>();

        let path = path.to_owned();
        thread::spawn(move || {
            let f = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(path)
                .unwrap();
            let mut w = BufWriter::new(f);
            loop {
                let x = rx.recv().expect("[WAL] error while receiving, closing file logger");
                let mut buf = bincode::serialize(&x).unwrap();
                let mut size = buf.len().to_le_bytes().to_vec();

                size.append(&mut buf);
                w.write(&*size).unwrap();
            }
        });

        WriteLog {
            inner,
            tx,
        }
    }
}

pub trait CacheMiddleware {}

