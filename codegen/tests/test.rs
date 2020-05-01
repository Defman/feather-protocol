use anyhow::Result;
use feather_protocol_spec::*;
use feather_protocol_codegen::ProtocolGenerator;
use std::collections::BTreeMap;
use indexmap::IndexMap;

#[test]
fn player_info_packet() -> Result<()> {
    let packet_bytes = include_bytes!("../../protocols/1.15.2.ron");
    // let packet_bytes = include_bytes!("./player_info_packet.ron");

    // let protocol_de: Protocol = ron::de::from_bytes(packet_bytes)?;

    let mut packets = BTreeMap::new();
    packets.insert(PacketIdentifier(
        PacketDirection::Client, PacketStage::Handshaking, 0.into()),
        CustomType::Struct {
            name: CustomTypeName::from("ma".to_owned()),
            fields: {
                let mut fields = IndexMap::new();
                fields.insert(FieldName::from("username".to_owned()), PrimitiveType::String(0).into());
                fields
            }
        }
    );

    let protocol = Protocol {
        version: ProtocolVersion::from(0),
        shared_types: vec![],
        packets,
    };

    let protocol = ProtocolGenerator::generate(protocol);

    println!("{}", protocol);

    Ok(())
}