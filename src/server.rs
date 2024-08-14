use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::time::SystemTime;

use mio::{Events, Interest, Poll, Token};
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};

use crate::cache::{Cache, CacheServer, middlewares};
use crate::cache::middlewares::{Middleware, MiddlewareNext};
use crate::cli::Args;
use crate::proto;
use crate::proto::{Frame, RequestCommand};

const SERVER: Token = Token(0);

pub fn start(args: &Args) -> Result<(), Box<dyn Error>> {
    let log = middlewares::Logger::new(args.verbose);
    let wal = middlewares::WriteLog::new(&args.wal.clone());
    let replicator = middlewares::Replicator::new(args.clone().replica);

    let mw: Vec<Box<dyn Middleware>> = vec![Box::new(&log), Box::new(&wal), Box::new(&replicator)];

    let cache = &Cache::new();

    wal.preload(&cache);

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(512);

    let addr = args.addr.parse()?;
    let mut server = TcpListener::bind(addr)?;
    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    let mut connections = HashMap::new();
    let mut client_token: Token = Token(SERVER.0 + 1);

    loop {
        let t = SystemTime::now();

        if let Err(err) = poll.poll(&mut events, None) {
            if interrupted(&err) {
                continue;
            }
            return Err(err.into());
        }

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    let (mut connection, address) = match server.accept() {
                        Ok((connection, address)) => (connection, address),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    };
                    connection.set_nodelay(true)?;

                    println!("Accepted connection from: {}", address);

                    let token = next(&mut client_token);
                    poll.registry()
                        .register(&mut connection, token, Interest::READABLE)?;

                    connections.insert(token, connection);
                },
                token => {
                    let done = if let Some(connection) = connections.get_mut(&token) {
                        handle_connection_event(connection, event, |x| {
                            MiddlewareNext::new(
                                &mut mw.iter().map(|mw| mw.as_ref()),
                                Box::new(|r| cache.on_request(r)),
                            )
                            .on_request(x)
                        })?
                    } else {
                        // Sporadic events happen, we can safely ignore them.
                        false
                    };
                    if done {
                        if let Some(mut connection) = connections.remove(&token) {
                            poll.registry().deregister(&mut connection)?;
                        }
                    }
                }
            }
        }
        println!("Event loop {:?}", t.elapsed());
    }
}

fn handle_connection_event<T: Fn(&RequestCommand) -> Vec<u8>>(
    connection: &mut TcpStream,
    event: &Event,
    cache: T,
) -> io::Result<bool> {
    if event.is_readable() {
        let mut c = connection.deref();

        loop {
            let request: Frame = match proto::deserialize(c) {
                Ok(Some(x)) => x,
                Err(ref err) if would_block(err) => break,
                Err(ref err) if interrupted(err) => continue,
                // Other errors we'll consider fatal.
                Err(err) => return Err(err),
                Ok(None) => {
                    println!("decoding resulted in disconnect");
                    return Ok(true);
                }
            };
            let res = cache(&request.clone().into());
            let buf: Vec<u8> = request.to_response(RequestCommand::Recv(res)).into();
            c.write(&*buf)?;
            // proto::encode(c, &)?;
        }
    }

    Ok(false)
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}
