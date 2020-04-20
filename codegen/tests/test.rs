use anyhow::Result;
use feather_protocol_spec::{PacketIdentifier, PacketDirection, PacketStage, Packet};
use feather_protocol_codegen::PacketGenerator;

#[test]
fn player_info_packet() -> Result<()> {
    let packet_bytes = include_bytes!("player_info_packet.ron");

    let packet_de: Packet = ron::de::from_bytes(packet_bytes)?;
    
    let packet_identifier = PacketIdentifier {
        direction: PacketDirection::ClientBound,
        stage: PacketStage::Play,
        id: 0.into(),
    };

    let (packet_tokens, _packet_ident) = PacketGenerator::generate(&packet_identifier, &packet_de);

    println!("{}", packet_tokens);

    Ok(())
}