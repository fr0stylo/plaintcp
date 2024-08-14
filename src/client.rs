use std::error::Error;
use std::ops::Deref;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use clap::Command;
use mio::net::TcpStream;
use promkit::preset::readline::Readline;
use promkit::suggest::Suggest;
use rand::distributions::{Alphanumeric, DistString};

use proto::Frame;
use regex::{Captures, Regex};

use crate::cli::Args;
use crate::proto;
use crate::proto::RequestCommand;

pub fn start(args: &Args) -> Result<(), Box<dyn Error>> {
    let addr = args.addr.clone();

    let con = std::net::TcpStream::connect(addr).unwrap();
    con.set_nodelay(true)?;

    loop {
        let str = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let t = SystemTime::now();
        proto::encode(&con, &Frame::new(RequestCommand::Set(str.clone(), str.clone().into_bytes())))?;
        let res = proto::decode(&con)?;
        println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        proto::encode(&con, &Frame::new(RequestCommand::Get(str.clone())))?;
        let res = proto::decode(&con)?;
        println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        proto::encode(&con, &Frame::new(RequestCommand::Delete(str.clone())))?;
        let res = proto::decode(&con)?;
        println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        sleep(Duration::from_millis(200));
    }
}

fn execute_request(con: &std::net::TcpStream, frame: &RequestCommand) -> Result<Option<RequestCommand>, std::io::Error> {
    let con = con.deref();
    proto::encode(con, &Frame::new(frame.clone()))?;
    let res = proto::decode(con)?;
    Ok(res)
}

pub fn interactive(args: &Args) -> Result<(), Box<dyn Error>> {
    let addr = args.addr.clone();

    let con = std::net::TcpStream::connect(addr).unwrap();
    con.set_nodelay(true)?;
    loop {
        let mut p = Readline::default()
            .enable_suggest(Suggest::from_iter([
                "GET",
                "SET",
                "DELETE",
            ]))
            .prompt()?;

        let res = p.run()?;
        let result = match res.as_str() {
            x if Regex::new("GET (.*)").unwrap().is_match(x) => {
                match Regex::new(r"GET (\w*)").unwrap().captures(x) {
                    Some(x) => {
                        let key = x.get(1).unwrap().as_str();
                        execute_request(&con, &RequestCommand::Get(key.to_owned())).expect("Failed to connect to remote")
                    }
                    None => None
                }
            }
            x  if Regex::new("SET (.*) (.*)").unwrap().is_match(x) => {
                match Regex::new(r"SET (\w*) (.*)").unwrap().captures(x) {
                    Some(x) => {
                        let key = x.get(1).unwrap().as_str();
                        let body = x.get(2).unwrap().as_str();
                        execute_request(&con, &RequestCommand::Set(key.to_owned(), body.as_bytes().into())).expect("Failed to connect to remote")
                    }
                    None => None
                }
            }
            x  if Regex::new("DELETE (.*)").unwrap().is_match(x) => {
                match Regex::new(r"DELETE (\w*)").unwrap().captures(x) {
                    Some(x) => {
                        let key = x.get(1).unwrap().as_str();
                        execute_request(&con, &RequestCommand::Delete(key.to_owned())).expect("Failed to connect to remote")
                    }
                    None => None
                }
            }
            _ => { None }
        };

        match result {
            None => {
                println!("Nothing was returned");
            }
            Some(x) => {
                println!("{x}");
            }
        }
    }
}