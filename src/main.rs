use std::error::Error;
use std::thread;

use clap::Parser;

use crate::cli::Args;

pub mod cli {
    use clap::Parser;

    #[derive(Parser, Debug, Clone)]
    #[command(version, about, long_about = None)]
    pub struct Args {
        #[arg(short, long, default_value_t = false)]
        pub server: bool,

        #[arg(short, long, default_value_t = ("127.0.0.1:9000").to_owned())]
        pub addr: String,
    }
}

pub mod proto;
pub mod cache;
pub mod client;
pub mod server;


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    env_logger::init();

    if args.server {
        return server::start(&args);
    }

    let a = args.clone();
    let t = thread::spawn(move || {
        client::start(&a).expect("Not working");
    });
    for _i in 0..10 {
        let a = args.clone();
        thread::spawn(move || {
            client::start(&a).expect("Not working");
        });
    }

    t.join().expect("TODO: panic message");
    Ok(())
}

