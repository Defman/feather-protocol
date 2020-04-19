use crate::types::{TryReadFrom, WriteInto};

pub trait Direction: Sized + 'static {}
pub struct ServerBound;
impl Direction for ServerBound {}
pub struct ClientBound;
impl Direction for ClientBound {}

pub trait Stage: Sized + 'static {}
pub struct HandshakeStage;
impl Stage for HandshakeStage {}
pub struct StatusStage;
impl Stage for StatusStage {}
pub struct LoginStage;
impl Stage for LoginStage {}
pub struct PlayStage;
impl Stage for PlayStage {}

/// Represents a packet.
pub trait Packet: Send + Sync + Sized + TryReadFrom + WriteInto {
    type Direction: Direction;
    type Stage: Stage;

    fn id(&self) -> u32;
    fn name(&self) -> &'static str;
}

pub trait Protocol: Sized + 'static {
    type ServerBoundHandshakePacket: Packet<Direction = ServerBound, Stage = HandshakeStage>;
    type ClientBoundHandshakePacket: Packet<Direction = ClientBound, Stage = HandshakeStage>;
    type ServerBoundStatusPacket: Packet<Direction = ServerBound, Stage = StatusStage>;
    type ClientBoundStatusPacket: Packet<Direction = ClientBound, Stage = StatusStage>; 
    type ServerBoundLoginPacket: Packet<Direction = ServerBound, Stage = LoginStage>; 
    type ClientBoundLoginPacket: Packet<Direction = ClientBound, Stage = LoginStage>; 
    type ServerBoundPlayPacket: Packet<Direction = ServerBound, Stage = PlayStage>; 
    type ClientBoundPlayPacket: Packet<Direction = ClientBound, Stage = PlayStage>;

    fn version() -> u64;
    fn minecraft_version() -> &'static str;
    fn minecraft_major_version() -> &'static str;
}
