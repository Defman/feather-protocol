use crate::types::{TryReadFrom, WriteInto};

mod direction {
    pub trait Direction: Sized + 'static {}
    pub struct Server;
    impl Direction for Server {}
    pub struct Client;
    impl Direction for Client {}
}

mod stage {
    pub trait Stage: Sized + 'static {}
    pub struct Handshake;
    impl Stage for Handshake {}
    pub struct Status;
    impl Stage for Status {}
    pub struct Login;
    impl Stage for Login {}
    pub struct Play;
    impl Stage for Play {}
}

/// Represents a packet.
pub trait Packet: Send + Sync + Sized + TryReadFrom + WriteInto {
    type Direction: direction::Direction;
    type Stage: stage::Stage;

    fn id(&self) -> u32;
    fn name(&self) -> &'static str;
}

pub trait Protocol: Sized + 'static {
    type ServerBoundHandshakePacket: Packet<Direction = direction::Server, Stage = stage::Handshake>;
    type ClientBoundHandshakePacket: Packet<Direction = direction::Client, Stage = stage::Handshake>;
    type ServerBoundStatusPacket: Packet<Direction = direction::Server, Stage = stage::Status>;
    type ClientBoundStatusPacket: Packet<Direction = direction::Client, Stage = stage::Status>; 
    type ServerBoundLoginPacket: Packet<Direction = direction::Server, Stage = stage::Login>; 
    type ClientBoundLoginPacket: Packet<Direction = direction::Client, Stage = stage::Login>; 
    type ServerBoundPlayPacket: Packet<Direction = direction::Server, Stage = stage::Play>; 
    type ClientBoundPlayPacket: Packet<Direction = direction::Client, Stage = stage::Play>;

    fn version() -> u64;
    fn minecraft_version() -> &'static str;
    fn minecraft_major_version() -> &'static str;
}
