use bytes::{Buf, BytesMut};
use thiserror::Error;

#[derive(Debug)]
pub enum Direction {
    Client,
    Server,
}

#[derive(Debug)]
pub enum Stage {
    Handshaking,
    Status,
    Login,
    Play,
}

/// Represents a packet.
pub trait Packet: Send + Sync + Sized {
    fn id(&self) -> u64;
    fn name(&self) -> &'static str;
    fn direction(&self) -> Direction;
    fn stage(&self) -> Stage;

    fn encode(&self) -> BytesMut;
    fn decode(buf: &mut impl Buf) -> Result<Self, DecodeError>;
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("TryRead: {0}")]
    TryRead(crate::types::Error),
    #[error("NonExistentPacket: ({direction:?}, {stage:?}, {id})")]
    NonExistentPacket {
        direction: Direction, 
        stage: Stage, 
        id: u64
    }
}

impl From<crate::types::Error> for DecodeError {
    fn from(err: crate::types::Error) -> Self {
        DecodeError::TryRead(err)
    }
}


pub trait Protocol: Sized + 'static {
    type ServerBoundHandshakePacket: Packet;
    type ClientBoundHandshakePacket: Packet;
    type ServerBoundStatusPacket: Packet;
    type ClientBoundStatusPacket: Packet; 
    type ServerBoundLoginPacket: Packet; 
    type ClientBoundLoginPacket: Packet; 
    type ServerBoundPlayPacket: Packet; 
    type ClientBoundPlayPacket: Packet;

    fn version() -> u64;
    fn minecraft_version() -> &'static str;
    fn minecraft_major_version() -> &'static str;
}
