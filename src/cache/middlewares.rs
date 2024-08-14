use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};
use std::thread;
use std::time::SystemTime;

use proto::encode;

use crate::proto;
use crate::proto::RequestCommand;

pub struct WriteLog {
    tx: Sender<RequestCommand>,
}

pub trait Middleware {
    fn on_request(&self, f: &RequestCommand, next: MiddlewareNext) -> Vec<u8> {
        next.on_request(f)
    }
}

pub struct MiddlewareNext<'a> {
    middlewares: &'a mut (dyn Iterator<Item=&'a dyn Middleware>),
    request_fn: Box<dyn FnOnce(&RequestCommand) -> Vec<u8> + 'a>,
}

impl<'a> MiddlewareNext<'a> {
    pub fn new(mw: &'a mut (dyn Iterator<Item=&'a dyn Middleware>), req: Box<dyn FnOnce(&RequestCommand) -> Vec<u8> + 'a>) -> Self {
        MiddlewareNext {
            middlewares: mw,
            request_fn: req,
        }
    }
    pub fn on_request(self, request: &RequestCommand) -> Vec<u8> {
        if let Some(step) = self.middlewares.next() {
            step.on_request(request, self)
        } else {
            (self.request_fn)(request)
        }
    }
}


impl Middleware for &WriteLog {
    fn on_request(&self, f: &RequestCommand, next: MiddlewareNext) -> Vec<u8> {
        match f {
            RequestCommand::Set(_, _) => {
                self.tx.send(f.clone()).expect("[WAL] Failed to send message for sink");
            }
            RequestCommand::Delete(_) => {
                self.tx.send(f.clone()).expect("[WAL] Failed to send message for sink");
            }
            _ => {}
        }

        next.on_request(f)
    }
}

impl WriteLog {
    pub fn new(path: &str) -> Self {
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
            tx,
        }
    }
}

pub struct Replicator {
    tx: SyncSender<RequestCommand>,
}


impl Middleware for &Replicator {
    fn on_request(&self, f: &RequestCommand, next: MiddlewareNext) -> Vec<u8> {
        match f {
            RequestCommand::Set(_, _) => {
                self.tx.send(f.clone()).expect("[Replicator] Failed to send message for sink");
            }
            RequestCommand::Delete(_) => {
                self.tx.send(f.clone()).expect("[Replicator] Failed to send message for sink");
            }
            _ => {}
        }

        next.on_request(f)
    }
}

impl Replicator {
    pub fn new(addrs: Vec<String>) -> Self {
        let addrs = addrs.clone();
        let (tx, rx) = sync_channel::<RequestCommand>(100);

        thread::spawn(move || {
            let mut replicas = HashMap::new();

            for addr in addrs {
                let s = TcpStream::connect(&addr).expect("[Replicator] connection failed");
                s.set_nodelay(true).unwrap();
                s.set_nonblocking(true).unwrap();

                replicas.insert(addr, s);
            }

            let mut i = 0;
            for x in rx.iter() {
                replicas.iter().clone().for_each(|(z, replica)| {
                    i = i + 1;
                    println!("socketAddr: {}", z);

                    encode(replica, &proto::Frame::new(x.clone())).expect("[Replicator] error while replicating");
                });
                println!("sent {}", i)
            }
        });

        Replicator {
            tx,
        }
    }
}


#[derive(Debug)]
pub struct Logger {
    verbose: bool,
}

impl Logger {
    pub fn new(verbose: bool) -> Self {
        Logger {
            verbose
        }
    }
}

impl Middleware for &Logger {
    fn on_request(&self, f: &RequestCommand, next: MiddlewareNext) -> Vec<u8> {
        let t = SystemTime::now();
        let res = next.on_request(f);
        if self.verbose {
            println!("[{:?}] {:?}", t.elapsed().unwrap(), f);
        }
        res
    }
}
