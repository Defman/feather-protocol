use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::LitStr;

use indexmap::IndexMap;
use std::collections::BTreeMap;

use feather_protocol_spec::{
    CustomType, EnumVariant, FieldName, Packet, PacketDirection, PacketIdentifier, PacketStage,
    Protocol, Type, VariantKey, ArrayLength,
};

pub struct ProtocolGenerator;

impl ProtocolGenerator {
    pub fn generate(protocol: Protocol) -> TokenStream {
        let mut packets = protocol.packets;
        let server_bound_packets = packets.split_off(&PacketIdentifier(
            PacketDirection::Server,
            PacketStage::Handshaking,
            0.into(),
        ));

        let client_bound_packets = packets.split_off(&PacketIdentifier(
            PacketDirection::Client,
            PacketStage::Handshaking,
            0.into(),
        ));

        let (client, _, _) =
            DirectionGenerator::generate(PacketDirection::Server, server_bound_packets);
        let (server, _, _) =
            DirectionGenerator::generate(PacketDirection::Client, client_bound_packets);

        let (shared_types_tokens, _shared_types_idents): (Vec<_>, Vec<_>) = protocol
            .shared_types
            .iter()
            .map(|(name, custom_type)| CustomTypeGenerator::generate(&*name, custom_type))
            .unzip();

        quote! {
            mod protocol {
                #client
                #server
            }

            mod shared {
                #(#shared_types_tokens)*
            }
        }
    }
}

struct DirectionGenerator;

impl DirectionGenerator {
    fn generate(
        direction: PacketDirection,
        mut packets: BTreeMap<PacketIdentifier, Packet>,
    ) -> (TokenStream, (Ident, Ident, Ident, Ident), Ident) {
        let direction_ident = Self::ident(direction);

        let play_packets =
            packets.split_off(&PacketIdentifier(direction, PacketStage::Play, 0.into()));

        let login_packets =
            packets.split_off(&PacketIdentifier(direction, PacketStage::Login, 0.into()));

        let status_packets =
            packets.split_off(&PacketIdentifier(direction, PacketStage::Status, 0.into()));

        let handshaking_packets = packets.split_off(&PacketIdentifier(
            direction,
            PacketStage::Handshaking,
            0.into(),
        ));

        let (handshaking, handshaking_ident) =
            StageGenerator::generate(direction, PacketStage::Handshaking, handshaking_packets);
        let (status, status_ident) =
            StageGenerator::generate(direction, PacketStage::Status, login_packets);
        let (login, login_ident) =
            StageGenerator::generate(direction, PacketStage::Login, status_packets);
        let (play, play_ident) =
            StageGenerator::generate(direction, PacketStage::Play, play_packets);

        (
            quote! {
                pub mod #direction_ident {
                    #handshaking
                    #status
                    #login
                    #play
                }
            },
            (handshaking_ident, status_ident, login_ident, play_ident),
            direction_ident,
        )
    }

    fn ident(direction: PacketDirection) -> Ident {
        match direction {
            PacketDirection::Client => Ident::new("client_bound", Span::call_site()),
            PacketDirection::Server => Ident::new("server_bound", Span::call_site()),
        }
    }
}

struct StageGenerator;

impl StageGenerator {
    pub fn generate(
        direction: PacketDirection,
        stage: PacketStage,
        packets: BTreeMap<PacketIdentifier, Packet>,
    ) -> (TokenStream, Ident) {
        use heck::CamelCase;
        let stage_ident = Self::ident(stage);

        let (packet_tokens, packets_idents): (Vec<_>, Vec<_>) = packets
            .iter()
            .map(|(identifier, packet)| PacketGenerator::generate(identifier, packet))
            .unzip();

        let packet_names: Vec<_> = packets
            .iter()
            .map(|(_, p)| &p.name)
            .map(|name| Ident::new(&name.to_camel_case(), Span::call_site()))
            .collect();

        let packet_ids: Vec<_> = packets
            .iter()
            .map(|(i, _ )| *i.id())
            .collect();

        let direction = match direction {
            PacketDirection::Client => {
                quote! { crate::Direction::Client }
            }
            PacketDirection::Server => {
                quote! { crate::Direction::Server }
            }
        };

        let stage = match stage {
            PacketStage::Handshaking => quote! { crate::Stage::Handshaking },
            PacketStage::Status => quote! { crate::Stage::Status },
            PacketStage::Login => quote! { crate::Stage::Login },
            PacketStage::Play => quote! { crate::Stage::Play },
        };

        (
            quote! {
                pub mod #stage_ident {
                    pub enum Packet {
                        #(#packet_names(#packets_idents)),*
                    }

                    impl crate::Packet for Packet {
                        fn id(&self) -> u64 {
                            match self {
                                #(
                                    Packet::#packet_names(packet) => packet.id(),
                                )*
                                _ => panic!(),
                            }
                        }

                        fn name(&self) -> &'static str {
                            match self {
                                #(
                                    Packet::#packet_names(packet) => packet.name(),
                                )*
                                _ => panic!()
                            }
                        }

                        fn direction(&self) -> crate::Direction {
                            #direction
                        }

                        fn stage(&self) -> crate::Stage {
                            #stage
                        }

                        fn encode(&self, buf: &mut bytes::BytesMut) -> usize {
                            match self {
                                #(
                                    Packet::#packet_names(packet) => packet.encode(buf),
                                )*
                                _ => 0,
                            }
                        }

                        fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::packet::DecodeError> {
                            use crate::types::*;
                            let mut packet_id = buf.slice(..5);
                            let packet_id: VarInt = packet_id.try_read()?;
                            let packet_id = *packet_id as u64; 
                            Ok(match packet_id {
                                #(
                                    #packet_ids => Packet::#packet_names(#packets_idents::decode(buf)?),
                                )*
                                _ => Err(crate::packet::DecodeError::NonExistentPacket {
                                    direction: #direction,
                                    stage: #stage,
                                    id: packet_id,
                                })?
                            })
                        }
                    }

                    #(
                        impl From<#packets_idents> for Packet {
                            fn from(packet: #packets_idents) -> Self {
                                Packet::#packet_names(packet)
                            }
                        }
                    )*

                    #(#packet_tokens)*
                }
            },
            stage_ident,
        )
    }

    fn ident(stage: PacketStage) -> Ident {
        match stage {
            PacketStage::Handshaking => Ident::new("handshaking", Span::call_site()),
            PacketStage::Status => Ident::new("status", Span::call_site()),
            PacketStage::Login => Ident::new("login", Span::call_site()),
            PacketStage::Play => Ident::new("play", Span::call_site()),
        }
    }
}

pub struct PacketGenerator;

impl PacketGenerator {
    pub fn generate(identifier: &PacketIdentifier, packet: &Packet) -> (TokenStream, Ident) {
        use heck::{SnakeCase, CamelCase};
        let ident = Ident::new(&*packet.name.to_snake_case(), Span::call_site());
        let packet_name = (*packet.name).to_camel_case();
        let (custom_type, custom_type_ident) =
            CustomTypeGenerator::generate(&packet_name, &packet.custom_type);

        let id_lit = *identifier.id();
        let name_lit = LitStr::new(&*packet.name, Span::call_site());

        let direction = match identifier.direction() {
            PacketDirection::Client => {
                quote! { crate::Direction::Client }
            }
            PacketDirection::Server => {
                quote! { crate::Direction::Server }
            }
        };

        let stage = match identifier.stage() {
            PacketStage::Handshaking => quote! { crate::Stage::Handshaking },
            PacketStage::Status => quote! { crate::Stage::Status },
            PacketStage::Login => quote! { crate::Stage::Login },
            PacketStage::Play => quote! { crate::Stage::Play },
        };

        let decode_custom_type = EncodingGenerator::decode_custom_type(&packet.custom_type);
        (
            quote! {
                pub use #ident::#custom_type_ident;
                pub mod #ident {
                    #custom_type

                    impl #custom_type_ident {
                        const ID: u64 = #id_lit;
                        const NAME: &'static str = #name_lit;
                        const DIRECTION: crate::Direction = #direction;
                        const STAGE: crate::Stage = #stage;
                    }

                    impl crate::Packet for #custom_type_ident {
                        fn id(&self) -> u64 {
                            Self::ID
                        }

                        fn name(&self) -> &'static str {
                            Self::NAME
                        }

                        fn direction(&self) -> crate::Direction {
                            Self::DIRECTION
                        }

                        fn stage(&self) -> crate::Stage {
                            Self::STAGE
                        }

                        fn encode(&self, buf: &mut bytes::BytesMut) -> usize {
                            use crate::types::*;
                            let mut total = 0;
                            total += VarInt::from(Self::ID as i32).write(buf);
                            total
                        }

                        fn decode(buf: &mut bytes::Bytes) -> Result<Self, crate::packet::DecodeError> {
                            use crate::types::*;
                            use bytes::Buf;
                            let mut packet_id = buf.slice(..5);
                            let packet_id: VarInt = packet_id.try_read()?;
                            let packet_id = *packet_id as u64; 
                            if packet_id != Self::ID {
                                Err(crate::packet::DecodeError::NonExistentPacket {
                                    direction: #direction,
                                    stage: #stage,
                                    id: packet_id,
                                })?;
                            }
                            Ok({
                                #decode_custom_type
                            })
                        }
                    }

                    // #serialization
                }
            },
            custom_type_ident,
        )
    }
}

pub struct EncodingGenerator;

impl EncodingGenerator {

    pub fn decode_custom_type(custom_type: &CustomType) -> TokenStream {
        match custom_type {
            CustomType::Struct(fields) => {
                let (field_names, field_decoders): (Vec<_>, Vec<_>) = fields
                    .iter()
                    .map(|(name, kind)| (
                        Self::field_ident(name),
                        Self::decode_type(kind),
                    )).unzip();
                quote! {
                    #(
                        let #field_names = #field_decoders;
                    )*
                    Self {
                        #(#field_names),*
                    }
                }
            },
            CustomType::Enum {
                variant,
                variants
            } => {
                let read_key = match variant {
                    EnumVariant::Prefixed(kind) => Self::decode_type(kind),
                    EnumVariant::Key(key) => Self::field_ident(key).to_token_stream(),
                };

                let (literals, names): (Vec<_>, Vec<_>) = variants
                    .iter()
                    .map(|(l, _)| (&l.0, CustomTypeGenerator::ident(&*l.1)))
                    .unzip();

                quote! {
                    
                }
            },
            _ => quote! { unimplemented!() }
        }
    }

    pub fn decode_type(ty: &Type) -> TokenStream {
        match ty {
            Type::Boolean => quote! { bool::try_read(buf)? },
            Type::U8 => quote! { u8::try_read(buf)? },
            Type::I8 => quote! { i8::try_read(buf)? },
            Type::U16 => quote! { u16::try_read(buf)? },
            Type::I16 => quote! { i16::try_read(buf)? },
            Type::U32 => quote! { u32::try_read(buf)? },
            Type::I32 => quote! { i32::try_read(buf)? },
            Type::U64 => quote! { u64::try_read(buf)? },
            Type::I64 => quote! { i64::try_read(buf)? },
            Type::F32 => quote! { f32::try_read(buf)? },
            Type::F64 => quote! { f64::try_read(buf)? },
            Type::VarInt => quote! { *VarInt::try_read(buf)? },
            Type::VarLong => quote! { *VarLong::try_read(buf)? },
            Type::Uuid => quote! { uuid::Uuid::try_read(buf)? },
            Type::String(_) => quote! { String::try_read(buf)? },
            Type::Nbt => quote! { Nbt::try_read(buf)?.take() },
            Type::Array { length, kind } => {
                let length = match length {
                    ArrayLength::FixedLength(length) => length.to_token_stream(),
                    ArrayLength::Key(name) => Self::field_ident(name).to_token_stream(),
                    ArrayLength::Prefixed(kind) => Self::decode_type(kind),
                    ArrayLength::RemainingLength => quote! { buf.remaining() },
                };
                let read_type =  EncodingGenerator::decode_type(kind);
                quote! {
                    {
                        let length = #length;
                        let mut array = Vec::with_capacity(length as usize);
                        for _ in 0..length {
                            array.push(#read_type)
                        }
                        array
                    }
                }
            }
            Type::Option(inner) => {
                let read_type = EncodingGenerator::decode_type(inner);
                quote! {
                    {
                        let some = bool::try_read(buf)?;
                        if some {
                            Some(#read_type)
                        } else {
                            None
                        }
                    }
                }
            },
            Type::CustomType(_, custom_type) => Self::decode_custom_type(custom_type),
            Type::Constant(_) => quote! { unimplemented!() },
            Type::Key(_) => quote! { unimplemented!() },
            Type::Shared(name) => quote! { unimplemented!() }
        }
    }
    
    pub fn field_ident(name: &str) -> Ident {
        use heck::SnakeCase;
        let name = name.to_snake_case();
        let name = match name.as_ref() {
            "type" => "kind",
            "match" => "match_",
            name => name,
        };
        Ident::new(&name, Span::call_site())
    }
}

struct CustomTypeGenerator;

impl CustomTypeGenerator {
    pub fn generate(name: &str, custom_type: &CustomType) -> (TokenStream, Ident) {
        match custom_type {
            CustomType::Struct(fields) => Self::generate_struct(name, fields),
            CustomType::Enum { variant, variants } => Self::generate_enum(name, variant, variants),
            CustomType::Unit => Self::generate_unit(name),
            CustomType::BitField(_) => (quote! {}, Self::ident(name)),
            CustomType::BitFlags { kind, flags } => Self::generate_bitflags(name, kind, flags),
        }
    }

    fn ident(name: &str) -> Ident {
        use heck::CamelCase;
        Ident::new(&name.to_camel_case(), Span::call_site())
    }

    fn generate_struct(name: &str, fields: &IndexMap<FieldName, Type>) -> (TokenStream, Ident) {
        use heck::SnakeCase;

        let custom_type_ident = Self::ident(name);
        let (field_names, field_types): (Vec<_>, Vec<_>) = fields
            .iter()
            .filter(|(_, ty)| !matches!(ty, Type::Key(_)))
            .map(|(name, ty)| {
                let name = name.to_snake_case();
                let name = match name.as_ref() {
                    "type" => "kind",
                    "match" => "match_",
                    name => name,
                };
                (
                    Ident::new(&name, Span::call_site()),
                    TypeGenerator::generate(ty),
                )
            })
            .unzip();

        let (filed_type_tokens, field_type_idents): (Vec<_>, Vec<_>) =
            field_types.into_iter().unzip();
        (
            quote! {
                pub struct #custom_type_ident {
                    #(#field_names: #field_type_idents),*
                }

                #(#filed_type_tokens)*
            },
            custom_type_ident,
        )
    }

    fn generate_enum(
        name: &str,
        _variant: &EnumVariant,
        variants: &BTreeMap<VariantKey, CustomType>,
    ) -> (TokenStream, Ident) {
        use heck::CamelCase;

        let custom_type_ident = Self::ident(name);

        let (variant_custom_type_tokens, variant_idents): (Vec<_>, Vec<_>) = variants
            .iter()
            .map(|(key, ty)| {
                let ident = Ident::new(&key.1.to_camel_case(), Span::call_site());
                match ty {
                    CustomType::Unit => (quote! {}, quote! { #ident }),
                    ty => {
                        let (custom_type_tokens, ty_ident) =
                            CustomTypeGenerator::generate(&key.1, ty);
                        (
                            custom_type_tokens,
                            quote! {
                                #ident(#ty_ident)
                            },
                        )
                    }
                }
            })
            .unzip();

        (
            quote! {
                pub enum #custom_type_ident {
                    #(#variant_idents),*,
                }

                #(#variant_custom_type_tokens)*
            },
            custom_type_ident,
        )
    }

    fn generate_unit(name: &str) -> (TokenStream, Ident) {
        let custom_type_ident = Self::ident(name);

        (
            quote! {
                pub struct #custom_type_ident;
            },
            custom_type_ident,
        )
    }

    fn generate_bitflags(
        name: &str,
        kind: &Type,
        flags: &BTreeMap<u64, String>,
    ) -> (TokenStream, Ident) {
        let name_ident = Self::ident(name);

        let (flag_idents, flag_values): (Vec<_>, Vec<_>) = flags
            .iter()
            .map(|(value, name)| {
                let flag_ident = Ident::new(&name.to_uppercase(), Span::call_site());
                (flag_ident, *value)
            })
            .unzip();

        let (_, kind) = TypeGenerator::generate(kind);

        (
            quote! {
                bitflags::bitflags! {
                    pub struct #name_ident: #kind {
                        #(const #flag_idents = #flag_values as #kind;)*
                    }
                }
            },
            name_ident,
        )
    }
}

pub struct TypeGenerator;

impl TypeGenerator {
    pub fn generate(ty: &Type) -> (TokenStream, TokenStream) {
        match ty {
            Type::Boolean => (quote! {}, quote! { bool }),
            Type::U8 => (quote! {}, quote! { u8 }),
            Type::I8 => (quote! {}, quote! { i8 }),
            Type::U16 => (quote! {}, quote! { u16 }),
            Type::I16 => (quote! {}, quote! { i16 }),
            Type::U32 => (quote! {}, quote! { u32 }),
            Type::I32 => (quote! {}, quote! { i32 }),
            Type::U64 => (quote! {}, quote! { u64 }),
            Type::I64 => (quote! {}, quote! { i64 }),
            Type::F32 => (quote! {}, quote! { f32 }),
            Type::F64 => (quote! {}, quote! { f64 }),
            Type::VarInt => (quote! {}, quote! { i32 }),
            Type::VarLong => (quote! {}, quote! { i64 }),
            Type::Uuid => (quote! {}, quote! { uuid::Uuid }),
            Type::String(_) => (quote! {}, quote! { String }),
            Type::Nbt => (quote! {}, quote! { nbt::Blob }),
            Type::Array { kind, .. } => {
                let (tokens, ident) = TypeGenerator::generate(kind.as_ref());
                (tokens, quote! { Vec<#ident> })
            }
            Type::Option(inner) => {
                let (tokens, ident) = TypeGenerator::generate(inner.as_ref());
                (tokens, quote! { Option<#ident> })
            }
            Type::CustomType(name, custom_type) => {
                let (tokens, ident) = CustomTypeGenerator::generate(name.as_ref(), &custom_type);
                (quote! { #tokens }, quote! { #ident })
            }
            Type::Constant(_) => (quote! {}, quote! {}),
            Type::Key(_) => (quote! {}, quote! {}),
            Type::Shared(name) => {
                use heck::CamelCase;
                let _name = Ident::new(&name.to_camel_case(), Span::call_site());
                (quote! {}, quote! { () })
            }
        }
    }
}
