use std::error::Error;
use std::io::Write;
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use promkit::preset::readline::Readline;
use promkit::suggest::Suggest;
use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;

use proto::Frame;

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
        proto::encode(
            &con,
            &Frame::new(RequestCommand::Set(str.clone(), str.clone().into_bytes())),
        )?;
        let res = proto::decode(&con)?;
        println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        let t = SystemTime::now();
        proto::encode(&con, &Frame::new(RequestCommand::Get(str.clone())))?;
        let res = proto::decode(&con)?;
        println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        // let t = SystemTime::now();
        // proto::encode(&con, &Frame::new(RequestCommand::Delete(str.clone())))?;
        // let res = proto::decode(&con)?;
        // println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        sleep(Duration::from_millis(200));
    }
}

fn execute_request(
    con: &std::net::TcpStream,
    frame: &RequestCommand,
) -> Result<Option<RequestCommand>, std::io::Error> {
    let mut con = con;
    let buf: Vec<u8> = Frame::new(frame.clone()).into();
    con.write(&*buf).expect("[Request] Error:");
    let res: Option<Frame> = proto::deserialize(con).expect("[Response] Error:");
    Ok(Some(res.unwrap().into()))
}

pub fn interactive(args: &Args) -> Result<(), Box<dyn Error>> {
    let addr = args.addr.clone();

    let con = std::net::TcpStream::connect(addr).unwrap();
    con.set_nodelay(true)?;
    let mut p = Readline::default()
        .enable_suggest(Suggest::from_iter(["GET", "SET", "DELETE", "KEYS"]))
        .enable_history()
        .prompt()?;

    loop {
        let res = p.run()?;
        let result = match res.as_str() {
            x if Regex::new("^GET").unwrap().is_match(x) => {
                match Regex::new(r"^GET (\w*)").unwrap().captures(x) {
                    Some(x) => {
                        let key = x.get(1).unwrap().as_str();
                        execute_request(&con, &RequestCommand::Get(key.to_owned()))
                            .expect("Failed to connect to remote")
                    }
                    None => {
                        println!("GET <key>");
                        None
                    }
                }
            }
            x if Regex::new("^SET").unwrap().is_match(x) => {
                match Regex::new(r"^SET (\w*) (.*)").unwrap().captures(x) {
                    Some(x) => {
                        let key = x.get(1).unwrap().as_str();
                        let body = x.get(2).unwrap().as_str();
                        execute_request(
                            &con,
                            &RequestCommand::Set(key.to_owned(), body.as_bytes().into()),
                        )
                        .expect("Failed to connect to remote")
                    }
                    None => {
                        println!("SET <key> <data>");
                        None
                    }
                }
            }
            x if Regex::new("^DELETE").unwrap().is_match(x) => {
                match Regex::new(r"^DELETE (\w*)").unwrap().captures(x) {
                    Some(x) => {
                        let key = x.get(1).unwrap().as_str();
                        execute_request(&con, &RequestCommand::Delete(key.to_owned()))
                            .expect("Failed to connect to remote")
                    }
                    None => {
                        println!("DELETE <key>");
                        None
                    }
                }
            }
            x if Regex::new(r"^KEYS").unwrap().is_match(x) => {
                match Regex::new(r"^KEYS (\d+) (\d+)").unwrap().captures(x) {
                    Some(x) => {
                        let take = x.get(1).unwrap().as_str();
                        let skip = x.get(2).unwrap().as_str();
                        execute_request(
                            &con,
                            &RequestCommand::Keys(
                                usize::from_str(take).unwrap(),
                                usize::from_str(skip).unwrap(),
                            ),
                        )
                        .expect("Failed to connect to remote")
                    }
                    None => {
                        println!("KEYS <take> <skip>");
                        None
                    }
                }
            }
            _ => None,
        };

        match result {
            None => {
                // println!("Nothing was returned");
            }
            Some(x) => {
                println!("{x}");
            }
        }
    }
}
