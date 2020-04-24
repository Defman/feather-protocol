pub mod packet;
mod codec;
pub mod types;

pub use packet::{Protocol, Packet, State, Direction, direction, Stage, stage};

/// Protocol version.
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolVersion {
    V1_13_2,
    V1_14_4,
    V1_15_0,
    V1_15_1,
    V1_15_2,
}

mod v1_15_2 {
    // feather_protocol_codegen_proc::protocol!("./protocols/1.15.2.ron");
}