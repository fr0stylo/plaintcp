use std::error::Error;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use proto::Frame;

use crate::cli::Args;
use crate::proto;
use crate::proto::RequestCommand;

pub fn start(args: &Args) -> Result<(), Box<dyn Error>> {
    let addr = args.addr.clone();

    let con = std::net::TcpStream::connect(addr).unwrap();
    con.set_nodelay(true)?;

    loop {
        let t = SystemTime::now();
        proto::encode(&con, &Frame::new(RequestCommand::Set("asdasdasda".to_owned(), [1, 213, 13, 21, 65, 165, 1].to_vec())))?;
        let res = proto::decode(&con)?;
        println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        proto::encode(&con, &Frame::new(RequestCommand::Get("asdasdasda".to_owned())))?;
        let res = proto::decode(&con)?;
        println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        proto::encode(&con, &Frame::new(RequestCommand::Del("asdasdasda".to_owned())))?;
        let res = proto::decode(&con)?;
        println!("Response ({:?}) : {:?}", t.elapsed().unwrap(), res.unwrap());
        sleep(Duration::from_millis(200));
    }
}
