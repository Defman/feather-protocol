use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{LitInt, LitStr};

use std::collections::BTreeMap;

use feather_protocol_spec::{Minecraft, Protocol, PacketDirection, PacketStage, PacketId, CustomType};

pub struct Generator {
    client_bound: DirectionGenerator,
    server_bound: DirectionGenerator,
}

pub struct DirectionGenerator {
    direction: PacketDirection,
    handshaking: StageGenerator,
    status: StageGenerator,
    login: StageGenerator,
    play: StageGenerator,
}

pub struct StageGenerator {
    stage: PacketStage,
    packets: BTreeMap<PacketId, CustomType>,
}

impl Generator {
    pub fn new(minecraft: Minecraft) {
        minecraft
    }

    pub fn generate(&self) {

    }
}


// pub fn generate(protocol: Protocol) -> TokenStream {
//     let packets: Vec<Packet> = protocol
//         .packets
//         .into_iter()
//         .map(|(_, packet)| packet)
//         .collect();

//     let (client_bound_packets, server_bound_packets) = packets
//         .into_iter()
//         .partition(|packet| packet.direction == PacketDirection::ToClient);
//     let server = generate_direction(PacketDirection::ToServer, server_bound_packets);
//     let client = generate_direction(PacketDirection::ToClient, client_bound_packets);

//     let version = protocol.protocol_version;
//     let minecraft_version = protocol.minecraft_version;
//     let minecraft_major_version = protocol.minecraft_major_version;

//     quote! {
//         pub struct Protocol;
//         impl crate::packets::Protocol for Protocol
//         {
//             type ServerBoundHandshakePacket = server_bound::handshake::Packet;
//             type ClientBoundHandshakePacket = client_bound::handshake::Packet;
//             type ServerBoundStatusPacket = server_bound::status::Packet;
//             type ClientBoundStatusPacket = client_bound::status::Packet;
//             type ServerBoundLoginPacket = server_bound::login::Packet;
//             type ClientBoundLoginPacket = client_bound::login::Packet;
//             type ServerBoundPlayPacket = server_bound::play::Packet;
//             type ClientBoundPlayPacket = client_bound::play::Packet;

//             fn version() -> u64 {
//                 #version
//             }

//             fn minecraft_version() -> &'static str {
//                 #minecraft_version
//             }

//             fn minecraft_major_version() -> &'static str {
//                 #minecraft_major_version
//             }
//         }

//         pub enum Packet {
//             ServerBound(server_bound::Packet),
//             ClientBound(client_bound::Packet)
//         }

//         impl Packet {
//             pub fn id(&self) -> u32 {
//                 // match self {
//                 //     Packet::ServerBound(packet) => crate::packets::Packet::id(packet),
//                 //     Packet::ClientBound(packet) => crate::packets::Packet::id(packet),
//                 // }
//                 unimplemented!()
//             }

//             pub fn name(&self) -> &'static str {
//                 // match self {
//                 //     Packet::ServerBound(packet) => crate::packets::Packet::name(packet),
//                 //     Packet::ClientBound(packet) => crate::packets::Packet::name(packet),
//                 // }
//                 unimplemented!()
//             }
//         }

//         impl From<server_bound::Packet> for Packet {
//             fn from(packet: server_bound::Packet) -> Self {
//                 Packet::ServerBound(packet)
//             }
//         }

//         impl From<client_bound::Packet> for Packet {
//             fn from(packet: client_bound::Packet) -> Self {
//                 Packet::ClientBound(packet)
//             }
//         }

//         impl crate::types::TryReadFrom for Packet {
//             fn try_read(buf: &mut impl bytes::Buf) -> Result<Self, crate::types::Error> {
//                 unimplemented!()
//             }
//         }

//         impl crate::types::WriteInto for Packet {
//             fn write(&self, buf: &mut impl bytes::BufMut) -> usize {
//                 unimplemented!()
//             }
//         }

//         #server
//         #client
//     }
// }

// fn generate_direction(direction: PacketDirection, packets: Vec<Packet>) -> TokenStream {
//     let side = ident(match direction {
//         PacketDirection::ToServer => "server_bound",
//         PacketDirection::ToClient => "client_bound",
//     });

//     let handshake_packets: Vec<_> = packets
//         .iter()
//         .filter(|packet| packet.category == PacketCategory::Handshake)
//         .collect();
//     let login_packets: Vec<_> = packets
//         .iter()
//         .filter(|packet| packet.category == PacketCategory::Login)
//         .collect();
//     let play_packets: Vec<_> = packets
//         .iter()
//         .filter(|packet| packet.category == PacketCategory::Play)
//         .collect();
//     let status_packets: Vec<_> = packets
//         .iter()
//         .filter(|packet| packet.category == PacketCategory::Status)
//         .collect();

//     let status = generate_enum(direction, PacketCategory::Status, status_packets);
//     let login = generate_enum(direction, PacketCategory::Login, login_packets);
//     let play = generate_enum(direction, PacketCategory::Play, play_packets);
//     let handshake = generate_enum(direction, PacketCategory::Handshake, handshake_packets);

//     quote! {
//         pub mod #side {
//             pub enum Packet {
//                 Handshake(handshake::Packet),
//                 Status(status::Packet),
//                 Login(login::Packet),
//                 Play(play::Packet),
//             }

//             impl Packet {
//                 pub fn id(&self) -> u32 {
//                     // match self {
//                     //     Packet::Handshake(packet) => crate::packets::Packet::id(packet),
//                     //     Packet::Status(packet) => crate::packets::Packet::id(packet),
//                     //     Packet::Login(packet) => crate::packets::Packet::id(packet),
//                     //     Packet::Play(packet) => crate::packets::Packet::id(packet),
//                     // }
//                     unimplemented!()
//                 }

//                 pub fn name(&self) -> &'static str {
//                     // match self {
//                     //     Packet::Handshake(packet) => crate::packets::Packet::name(packet),
//                     //     Packet::Status(packet) => crate::packets::Packet::name(packet),
//                     //     Packet::Login(packet) => crate::packets::Packet::name(packet),
//                     //     Packet::Play(packet) => crate::packets::Packet::name(packet),
//                     // }
//                     unimplemented!()
//                 }
//             }

//             impl crate::types::TryReadFrom for Packet {
//                 fn try_read(buf: &mut impl bytes::buf::Buf) -> Result<Self, crate::types::Error> {
//                     unimplemented!()
//                 }
//             }

//             impl crate::types::WriteInto for Packet {
//                 fn write(&self, buf: &mut impl bytes::buf::BufMut) -> usize {
//                     unimplemented!()
//                 }
//             }

//             #handshake
//             #status
//             #login
//             #play
//         }
//     }
// }

// fn generate_enum(
//     direction: PacketDirection,
//     category: PacketCategory,
//     packets: Vec<&Packet>,
// ) -> TokenStream {
//     let state = ident(match category {
//         PacketCategory::Handshake => "handshake",
//         PacketCategory::Status => "status",
//         PacketCategory::Login => "login",
//         PacketCategory::Play => "play",
//     });

//     let (packets, packet_names): (Vec<TokenStream>, Vec<Ident>) =
//         packets.into_iter().map(generate_packet).unzip();

//     let bound = match direction {
//         PacketDirection::ToServer => quote! { crate::packets::ServerBound },
//         PacketDirection::ToClient => quote! { crate::packets::ClientBound },
//     };

//     let stage = match category {
//         PacketCategory::Handshake => quote! { crate::packets::HandshakeStage },
//         PacketCategory::Status => quote! { crate::packets::StatusStage },
//         PacketCategory::Login => quote! { crate::packets::LoginStage },
//         PacketCategory::Play => quote! { crate::packets::PlayStage },
//     };

//     quote! {
//         pub mod #state {
//             pub enum Packet {
//                 #(
//                     #packet_names(#packet_names)
//                 ),*
//             }

//             impl crate::packets::Packet for Packet {
//                 type Direction = #bound;
//                 type Stage = #stage;

//                 fn id(&self) -> u32 {
//                     match self {
//                         #(
//                             Packet::#packet_names(packet) => packet.id()
//                         ),*
//                     }
//                 }

//                 fn name(&self) -> &'static str {
//                     match self {
//                         #(
//                             Packet::#packet_names(packet) => packet.name()
//                         ),*
//                     }
//                 }
//             }

//             impl crate::types::TryReadFrom for Packet {
//                 fn try_read(buf: &mut impl bytes::buf::Buf) -> Result<Self, crate::types::Error> {
//                     unimplemented!()
//                 }
//             }

//             impl crate::types::WriteInto for Packet {
//                 fn write(&self, buf: &mut impl bytes::buf::BufMut) -> usize {
//                     unimplemented!()
//                 }
//             }

//             #(
//                 #packets
//             )*
//         }
//     }
// }

// fn generate_packet(packet: &Packet) -> (TokenStream, Ident) {
//     let packet_name: Ident = ident(&packet.name);
//     let packet_id: LitInt = LitInt::new(&packet.id.to_string(), Span::call_site());
//     let packet_identifier: LitStr = LitStr::new(&packet.identifier, Span::call_site());
//     let packet_fields_names: Vec<Ident> = packet.fields.keys().map(|k| ident(k)).collect();
//     let packet_fields_types: Vec<TokenStream> = packet
//         .fields
//         .values()
//         .map(|ty| tokenize_field_type(ty))
//         .collect();

//     let read_packet = read_packet(packet);

//     let bound = match packet.direction {
//         PacketDirection::ToServer => quote! { crate::packets::ServerBound },
//         PacketDirection::ToClient => quote! { crate::packets::ClientBound },
//     };

//     let stage = match packet.category {
//         PacketCategory::Handshake => quote! { crate::packets::HandshakeStage },
//         PacketCategory::Status => quote! { crate::packets::StatusStage },
//         PacketCategory::Login => quote! { crate::packets::LoginStage },
//         PacketCategory::Play => quote! { crate::packets::PlayStage },
//     };

//     (
//         quote! {
//             pub struct #packet_name {
//                 #(
//                     pub #packet_fields_names: #packet_fields_types
//                 ),*
//             }

//             impl crate::packets::Packet for #packet_name {
//                 type Direction = #bound;
//                 type Stage = #stage;

//                 fn id(&self) -> u32 {
//                     #packet_id
//                 }

//                 fn name(&self) -> &'static str {
//                     #packet_identifier
//                 }
//             }

//             #read_packet

//             impl crate::types::WriteInto for #packet_name {
//                 fn write(&self, buf: &mut impl bytes::buf::BufMut) -> usize {
//                     // use crate::types::WriteInto;
//                     // let mut total = 0;
//                     // #(
//                     //     total += self.#packet_fields_names.write(buf);
//                     // )*
//                     // total
//                     unimplemented!()
//                 }
//             }
//         },
//         packet_name,
//     )
// }

// fn read_packet(packet: &Packet) -> TokenStream {
//     let packet_name: Ident = ident(&packet.name);

//     let read_fields: Vec<TokenStream> = packet
//         .fields
//         .iter()
//         .map(|(name, ty)| read_field(name, ty))
//         .collect();

//     let packet_fields_names: Vec<Ident> = packet.fields.keys().map(|k| ident(k)).collect();

//     quote! {
//         impl crate::types::TryReadFrom for #packet_name {
//             fn try_read(buf: &mut impl bytes::buf::Buf) -> Result<Self, crate::types::Error> {
//                 #(
//                     #read_fields
//                 )*

//                 Ok(Self {
//                     #(
//                         #packet_fields_names
//                     ),*
//                 })
//             }
//         }
//     }
// }

// fn write_packet(packet: &Packet) -> TokenStream {
//     unimplemented!();
// }

// fn read_field(name: &FieldName, ty: &FieldType) -> TokenStream {
//     let name = ident(name);
//     let kind = tokenize_field_type(ty);

//     match ty {
//         FieldType::Uuid
//         | FieldType::Varint
//         | FieldType::U8
//         | FieldType::U16
//         | FieldType::I8
//         | FieldType::I16
//         | FieldType::I32
//         | FieldType::I64
//         | FieldType::F32
//         | FieldType::F64
//         | FieldType::Bool => quote! { 
//             let #name: #kind = crate::types::TryReadFrom::try_read(buf)?; },
//         FieldType::Switch {
//             compare_to,
//             fields,
//         } => {
//             let compare_to = ident(compare_to);
//             let fields: Vec<TokenStream> = fields.iter().map(|(name, ty)| read_field(name, ty)).collect();
//             quote! { }
//         },
//         FieldType::Bitfield {
//             fields
//         } => {
//             let field_names: Vec<Ident> = fields.keys().map(|k| ident(k)).collect();
//             let field_sizes: Vec<u32> = fields.values().map(|(size, _)| size).copied().collect();
//             let field_offsets: Vec<u32> = field_sizes
//                 .iter()
//                 .scan(0, |acc, size| {
//                     let offset = acc.clone();
//                     *acc += size;
//                     Some(offset)
//                 })
//                 .collect();
//             let signed: Vec<TokenStream> = fields.values().map(|(_, signed)| if *signed {
//                     quote! {
//                         as i64
//                     }
//                 } else {
//                     quote! {
//                         as i32
//                     }
//                 }).collect();
//             quote! {
//                 let val: u64 = buf.try_read()?;
//                 #(
//                     let #field_names = ((val & (std::u64::MAX >> (64 - size) << (64 - size - offset))) >> (64 - size - offset)) #signed;
//                 )*
//             }
//         }
//         _ => quote! { },
//     }
// }

// // 0b110100
// // 0b111111 >> 

// fn tokenize_field_type(ty: &FieldType) -> TokenStream {
//     match ty {
//         FieldType::Uuid => quote! { uuid::Uuid },
//         FieldType::Varint => quote! { crate::types::VarInt },
//         FieldType::U8 => quote! { u8 },
//         FieldType::U16 => quote! { u16 },
//         FieldType::I8 => quote! { i8 },
//         FieldType::I16 => quote! { i16 },
//         FieldType::I32 => quote! { i32 },
//         FieldType::I64 => quote! { i64 },
//         FieldType::F32 => quote! { f32 },
//         FieldType::F64 => quote! { f64 },
//         FieldType::Bool => quote! { bool },
//         FieldType::Mapper { mappings } => quote! { () },
//         FieldType::Pstring { .. } => quote! { String },
//         FieldType::Buffer => quote! { Vec<u8> },
//         FieldType::Option { of } => {
//             let inner = tokenize_field_type(of);
//             quote! { Option<#inner> }
//         }
//         FieldType::EntityMetadataLoop { .. } => quote! { std::collections::BTreeMap<u8, ()> },
//         FieldType::Bitfield { fields } => {
//             let names: Vec<Ident> = fields.keys().map(ident).collect();
//             let types: Vec<TokenStream> fields.values().map(|(_, signed)| if signed { quote! { i64 } } else { quote! { u64 } });
//             quote! { 
                
//             }
//         },
//         FieldType::Container { .. } => quote! { () },
//         FieldType::Switch {
//             compare_to: _,
//             fields: _,
//         } => quote! { u8 },
//         FieldType::Void => quote! { u8 },
//         FieldType::Array {
//             count_type: _,
//             of: _,
//         } => quote! { u8 },
//         FieldType::RestBuffer => quote! { u8 },
//         FieldType::OptionalNbt => quote! { crate::types::Nbt<nbt::Blob> },
//         FieldType::Nbt => quote! { crate::types::Nbt<nbt::Blob> },
//         FieldType::Custom { name: _ } => quote! { u8 },
//     }
// }

// fn ident(s: impl AsRef<str>) -> Ident {
//     let ident = match s.as_ref() {
//         "type" => "kind",
//         ident => ident,
//     };
//     Ident::new(ident, Span::call_site())
// }
