mod packets;
mod codec;
pub mod types;

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
