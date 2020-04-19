use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum PacketStage {
    Handshaking,
    Status,
    Login,
    Play,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum PacketDirection {
    ServerBound,
    ClientBound,
}

macro_rules! wrap {
    ($outer:ident, $inner:ident) => {
        impl std::ops::Deref for $outer {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PacketId(u64);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PacketName(String);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MinecraftVersion(String);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProtocolVersion(u64);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SharedTypeId(String);

wrap!(PacketId, u64);
wrap!(PacketName, String);
wrap!(MinecraftVersion, String);
wrap!(ProtocolVersion, u64);
wrap!(SharedTypeId, String);


#[derive(Debug, Clone, Ord, Eq, Serialize, Deserialize)]
pub struct PacketIdentifier {
    pub stage: PacketStage,
    pub direction: PacketDirection,
    pub id: PacketId,
    pub name: PacketName,
}

impl PartialEq<Self> for PacketIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.stage.eq(&other.stage) && self.direction.eq(&other.direction) && self.id.eq(&other.id)
    }
}

impl PartialOrd<Self> for PacketIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.stage.cmp(&other.stage).then(
                self.direction
                    .cmp(&other.direction)
                    .then(self.id.cmp(&other.id)),
            ),
        )
    }
}
