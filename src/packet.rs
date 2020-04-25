use bytes::{Bytes, BytesMut};
use std::marker::PhantomData;
use thiserror::Error;

pub mod direction {
    pub trait Direction: std::fmt::Debug + 'static {}
    #[derive(Debug, PartialEq, Eq)]
    pub struct Client;
    impl Direction for Client {}
    #[derive(Debug, PartialEq, Eq)]
    pub struct Server;
    impl Direction for Server {}
}

pub mod stage {
    pub trait Stage: std::fmt::Debug + 'static {}
    #[derive(Debug, PartialEq, Eq)]
    pub struct Handshaking;
    impl Stage for Handshaking {}
    #[derive(Debug, PartialEq, Eq)]
    pub struct Status;
    impl Stage for Status {}
    #[derive(Debug, PartialEq, Eq)]
    pub struct Login;
    impl Stage for Login {}
    #[derive(Debug, PartialEq, Eq)]
    pub struct Play;
    impl Stage for Play {}
}

pub use direction::Direction;
pub use stage::Stage;

pub trait PacketEnum<D: Direction, S: Stage> {
    fn packet_id(&self) -> u64;
    fn packet_name(&self) -> &'static str;
}

/// Represents a packet.
pub trait Packet<D: Direction, S: Stage>: Send + Sync + Sized {
    const ID: usize;
    const NAME: &'static str;

    fn encode(&self, buf: &mut BytesMut) -> usize;
    fn decode(buf: &mut Bytes) -> Result<Self, DecodeError>;
}

pub trait State<D: Direction, S: Stage> {
    type Packet: PacketEnum<D, S>;
}

pub trait Protocol: Sized + 'static {
    fn version() -> u64;
    fn minecraft_version() -> &'static str;
    fn minecraft_major_version() -> &'static str;
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("TryRead: {0}")]
    TryRead(crate::types::Error),
    #[error("NonExistentPacket: ({direction:?}, {stage:?}, {id})")]
    NonExistentPacket {
        direction: PhantomData<dyn Direction>,
        stage: PhantomData<dyn Direction>,
        id: u64,
    },
}

impl From<crate::types::Error> for DecodeError {
    fn from(err: crate::types::Error) -> Self {
        DecodeError::TryRead(err)
    }
}
