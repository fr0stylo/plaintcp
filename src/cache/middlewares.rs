use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::SystemTime;
use proto::encode;

use crate::cache::CacheServer;
use crate::proto;
use crate::proto::RequestCommand;

#[derive(Debug)]
pub struct Logger<S> where S: CacheServer {
    verbose: bool,
    inner: S,
}

impl<S> Logger<S> where S: CacheServer {
    pub fn new(verbose: bool, inner: S) -> Self {
        Logger {
            verbose,
            inner,
        }
    }
}

impl<S> CacheServer for Logger<S> where S: CacheServer {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8> {
        if self.verbose {
            println!("{:?}", f);
        }
        self.inner.on_request(f)
    }
}

impl<S> CacheServer for &Logger<S> where S: CacheServer {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8> {
        let t = SystemTime::now();
        let res = self.inner.on_request(f);
        if self.verbose {
            println!("[{:?}] {:?}", t.elapsed().unwrap(), f);
        }
        res
    }
}

pub struct WriteLog<S: CacheServer> {
    inner: S,
    tx: Sender<RequestCommand>,
}

impl<S: CacheServer> CacheServer for &WriteLog<S> {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8> {
        match f {
            RequestCommand::Set(_, _) => {
                self.tx.send(f.clone()).expect("[WAL] Failed to send message for sink");
            }
            RequestCommand::Delete(_) => {
                self.tx.send(f.clone()).expect("[WAL] Failed to send message for sink");
            }
            _ => {}
        }

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
                w.flush().unwrap();
            }
        });

        WriteLog {
            inner,
            tx,
        }
    }
}

pub struct Replicator<S> where S: CacheServer {
    inner: S,
    tx: Sender<RequestCommand>,
}

impl<S> CacheServer for Replicator<S> where S: CacheServer {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8> {
        self.tx.send(f.clone()).expect("[Replicator] Failed to send message for sink");

        self.inner.on_request(f)
    }
}

impl<S> CacheServer for &Replicator<S> where S: CacheServer {
    fn on_request(&self, f: &RequestCommand) -> Vec<u8> {
        match f {
            RequestCommand::Set(_, _) => {
                self.tx.send(f.clone()).expect("[Replicator] Failed to send message for sink");
            }
            RequestCommand::Delete(_) => {
                self.tx.send(f.clone()).expect("[Replicator] Failed to send message for sink");
            }
            _ => {}
        }

        self.inner.on_request(f)
    }
}

impl<S> Replicator<S> where S: CacheServer {
    pub fn new(addrs: Vec<String>, inner: S) -> Self {
        let addrs = addrs.clone();
        let (tx, rx) = channel::<RequestCommand>();

        thread::spawn(move || {
            let mut replicas = HashMap::new();

            for addr in addrs {
                let s = TcpStream::connect(&addr).expect("[Replicator] connection failed");

                replicas.insert(addr, s);
            }

            loop {
                let x = rx.recv().expect("[Replicator] error while receiving, closing ");
                println!("Sending {:?}", x);
                replicas.iter().clone().for_each(|(_, replica)| {
                    encode(replica, &proto::Frame::new(x.clone())).expect("[Replicator] error while replicating");
                })
            }
        });

        Replicator {
            inner,
            tx,
        }
    }
}

pub trait CacheMiddleware {}

