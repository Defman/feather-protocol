use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum PacketStage {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Play = 3,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum PacketDirection {
    Client = 0,
    Server = 1,
}

macro_rules! wrap {
    ($outer:ident, $inner:ident) => {
        impl std::ops::Deref for $outer {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$inner> for $outer {
            fn from(inner: $inner) -> Self {
                Self(inner)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PacketId(u64);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PacketName(String);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MinecraftVersion(String);

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProtocolVersion(u64);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SharedTypeId(String);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariantName(String);

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct FieldName(String);


wrap!(PacketId, u64);
wrap!(PacketName, String);
wrap!(MinecraftVersion, String);
wrap!(ProtocolVersion, u64);
wrap!(SharedTypeId, String);
wrap!(VariantName, String);
wrap!(FieldName, String);


#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketIdentifier(pub PacketDirection, pub PacketStage, pub PacketId);

impl PacketIdentifier {
    pub fn direction(&self) -> PacketDirection {
        self.0
    }

    pub fn stage(&self) -> PacketStage {
        self.1
    }
    
    pub fn id(&self) -> PacketId {
        self.2
    }
}