use std::error::Error;

use clap::Parser;
use cache::{Cache, middlewares};

use crate::cli::Args;

pub mod cli {
    use clap::Parser;

    #[derive(Parser, Debug)]
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
        let cache = Cache::new();
        let middlewared = middlewares::Logger::new(&cache);

        return server::start(&args, &middlewared);
    }

    client::start(&args)
}

