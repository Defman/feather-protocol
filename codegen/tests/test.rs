use anyhow::Result;
use feather_protocol_spec::Protocol;
use feather_protocol_codegen::ProtocolGenerator;

#[test]
fn player_info_packet() -> Result<()> {
    let packet_bytes = include_bytes!("../../protocols/1.15.2.ron");

    let protocol_de: Protocol = ron::de::from_bytes(packet_bytes)?;

    let protocol = ProtocolGenerator::generate(protocol_de);

    println!("{}", protocol);

    Ok(())
}