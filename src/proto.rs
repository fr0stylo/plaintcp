use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Write};
use std::mem::size_of;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

const VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RequestCommand {
    Empty,
    Get(String),
    Set(String, Vec<u8>),
    Delete(String),
    Keys(usize, usize),

    Error(Vec<u8>),
    Recv(Vec<u8>),
}

impl Into<Vec<u8>> for RequestCommand {
    fn into(self) -> Vec<u8> {
        encode_vec(self)
    }
}

impl Default for RequestCommand {
    fn default() -> Self {
        RequestCommand::Empty
    }
}

impl Display for RequestCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestCommand::Empty => { write!(f, "EMPTY") }

            RequestCommand::Get(key) => { write!(f, "GET {}", key) }
            RequestCommand::Set(key, body) => { write!(f, "SET {}, {}", key, String::from_utf8(body.clone()).unwrap()) }
            RequestCommand::Delete(key) => { write!(f, "DELETE {}", key) }
            RequestCommand::Keys(take, skip) => { write!(f, "KEYS {} {}", take, skip) }

            RequestCommand::Error(error) => { write!(f, "ERROR {}", String::from_utf8(error.clone()).unwrap()) }
            RequestCommand::Recv(buf) => { write!(f, "<< {}", String::from_utf8(buf.clone()).unwrap()) }
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Frame {
    version: u8,
    id: u64,
    command: RequestCommand,
}

impl Into<Vec<u8>> for Frame {
    fn into(self) -> Vec<u8> {
        encode_vec(self)
    }
}

impl Into<RequestCommand> for Frame {
    fn into(self) -> RequestCommand {
        self.command
    }
}

impl Frame {
    pub fn new(command: RequestCommand) -> Self {
        Self {
            version: VERSION,
            id: 1,
            command,
        }
    }

    pub fn to_response(&self, command: RequestCommand) -> Self {
        Self {
            version: self.version,
            id: self.id,
            command,
        }
    }
}

pub fn decode<T: Read>(mut r: T) -> Result<Option<RequestCommand>, std::io::Error> {
    let mut buf = [0u8; size_of::<usize>()];

    let i = r.read(&mut buf)?;
    if i == 0 {
        return Ok(None);
    }
    let size = usize::from_le_bytes(buf);
    let mut buf = vec![0u8; size];
    let i = r.read(&mut buf)?;
    if i == 0 {
        return Ok(None);
    }

    let frame = bincode::deserialize::<Frame>(&*buf).unwrap();
    Ok(Some(frame.command))
}


pub fn deserialize<T, R>(mut r: T) -> Result<Option<R>, std::io::Error> where T: Read, R: DeserializeOwned {
    let mut buf = [0u8; size_of::<usize>()];

    let i = r.read(&mut buf)?;
    if i == 0 {
        return Ok(None);
    }
    let size = usize::from_le_bytes(buf);
    let mut buf = vec![0u8; size];
    let i = r.read(&mut buf)?;
    if i == 0 {
        return Ok(None);
    }

    Ok(Some(bincode::deserialize::<R>(&*buf).unwrap()))
}

pub fn encode_vec<T: Serialize>(f: T) -> Vec<u8> {
    let mut buf = bincode::serialize(&f).unwrap();
    let mut size = buf.len().to_le_bytes().to_vec();

    size.append(&mut buf);
    size
}

pub fn encode<T: Write>(mut w: T, f: &Frame) -> Result<usize, std::io::Error> {
    let mut buf = bincode::serialize(f).unwrap();
    let mut size = buf.len().to_le_bytes().to_vec();

    size.append(&mut buf);

    let i = w.write(&*size)?;
    Ok(i)
}
