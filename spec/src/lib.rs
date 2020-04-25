use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use indexmap::IndexMap;

mod primitives;
pub use primitives::*;

mod custom_type;
pub use custom_type::*;

mod validation;
pub use validation::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minecraft {
    pub version: MinecraftVersion,
    pub major_version: MinecraftVersion,
    pub protocol: Protocol,
}
/// Defines all packets and field types for a protocol version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub version: ProtocolVersion,
    /// The set of all packets defined for this protocol.
    ///
    /// Keys in this map are (packet_direction, packet_stage, packet_id, packet_name).
    pub packets: BTreeMap<PacketIdentifier, CustomType>,

    /// Types which are shared across multiple packets.
    pub shared_types: Vec<CustomType>,
}