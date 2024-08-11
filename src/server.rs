use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::ops::Deref;
use std::time::SystemTime;

use mio::{Events, Interest, Poll, Token};
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};

use crate::cache::CacheServer;
use crate::cli::Args;
use crate::proto;
use crate::proto::RequestCommand;

const SERVER: Token = Token(0);

pub fn start<S: CacheServer>(args: &Args, cache: S) -> Result<(), Box<dyn Error>> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

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
                    poll.registry().register(
                        &mut connection,
                        token,
                        Interest::READABLE,
                    )?;

                    connections.insert(token, connection);
                }
                token => {
                    let done = if let Some(connection) = connections.get_mut(&token) {
                        handle_connection_event(connection, event, &bootstrapedCache)?
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


fn handle_connection_event<T: CacheServer>(
    connection: &mut TcpStream,
    event: &Event,
    cache: T,
) -> io::Result<bool> {
    if event.is_readable() {
        let c = connection.deref();

        let mut command: RequestCommand = RequestCommand::default();

        loop {
            command = match proto::decode(c) {
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
            break;
        }

        let res = cache.on_request(&command);
        proto::encode(c, &proto::Frame::new(RequestCommand::Recv(res)))?;
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
