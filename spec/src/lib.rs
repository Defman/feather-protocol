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
    pub packets: BTreeMap<PacketIdentifier, Packet>,

    /// Types which are shared across multiple packets.
    pub shared_types: BTreeMap<SharedTypeId, CustomType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub name: PacketName,
    pub custom_type: CustomType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn slot() -> Result<()> {
        let player_info = r#"
            Struct({
                "action": Key(VarInt),
                "players": Array (
                    length: Prefixed(VarInt),
                    schema: CustomType("player", Struct({
                        "uuid": Uuid,
                        "action": CustomType("action", Enum(
                            variant: Key("action"),
                            variants: {
                                (0, "add_player"): Struct({
                                    "name": String(16),
                                    "properties": Array(
                                        length: Prefixed(VarInt),
                                        schema: CustomType("property", Struct({
                                            "name": String(32767),
                                            "value": String(32767),
                                            "is_signed": Boolean,
                                            "signature": Option(String(32767)),
                                        })),
                                    ),
                                    "gamemode": VarInt,
                                    "ping": VarInt,
                                    "display_name": Option(String(32767)),
                                }),
                                (1, "update_gamemode"): Struct({
                                    "gamemode": VarInt,
                                }),
                                (2, "update_latency"): Struct({
                                    "ping": VarInt,
                                }),
                                (3, "update_display_name"): Struct({
                                    "display_name": Option(String(32767)),
                                }),
                                (4, "remove_player"): Unit,
                            },
                        )),
                    }))
                ),
            })
        "#;

        let player_info_de: CustomType = ron::de::from_str(&player_info)?;
        let player_info_se = ron::ser::to_string_pretty(&player_info_de, Default::default())?;

        println!("{:?}", player_info_de);
        println!("{}", player_info_se);

        Ok(())
    }

    #[test]
    fn combat_event() -> Result<()> {
        let combat_packet = CustomType::Enum {
            variant: EnumVariant::Prefixed(Box::new(Type::VarInt)),
            variants: {
                let mut variants = BTreeMap::new();
                variants.insert(VariantKey(Literal::Int(0), "enter_combat".to_owned().into()), CustomType::Unit);
                variants.insert(VariantKey(Literal::Int(1), "end_combat".to_owned().into()), CustomType::Struct({
                    let mut fields = IndexMap::new();
                    fields.insert("duration".to_owned().into(), Type::VarInt);
                    fields.insert("entity_id".to_owned().into(), Type::I32);
                    fields
                }));
                variants.insert(VariantKey(Literal::Int(2), "entity_dead".to_owned().into()), CustomType::Struct({
                    let mut fields = IndexMap::new();
                        fields.insert("player_id".to_owned().into(), Type::VarInt);
                        fields.insert("entity_id".to_owned().into(), Type::I32);
                        fields.insert("message".to_owned().into(), Type::String(32767));
                        fields
                }));
                variants
            }
        };

        let combat_packet_ser = ron::ser::to_string_pretty(&combat_packet, Default::default())?;

        println!("{}", combat_packet_ser);

        Ok(())
    }
}
