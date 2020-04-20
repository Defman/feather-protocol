use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{LitInt, LitStr};

use indexmap::IndexMap;
use std::collections::BTreeMap;

use feather_protocol_spec::{
    CustomType, EnumVariant, FieldName, Literal, Minecraft, Packet, PacketDirection, PacketId,
    PacketIdentifier, PacketStage, Protocol, Type, VariantKey,
};

pub struct ProtocolGenerator;

impl ProtocolGenerator {
    pub fn generate(protocol: Protocol) -> TokenStream {
        let mut packets = protocol.packets;
        let server_bound_packets = packets.split_off(&PacketIdentifier {
            direction: PacketDirection::ServerBound,
            stage: PacketStage::Handshaking,
            id: 0.into(),
        });

        let client_bound_packets = packets.split_off(&PacketIdentifier {
            direction: PacketDirection::ClientBound,
            stage: PacketStage::Handshaking,
            id: 0.into(),
        });

        let (client, _, _) = DirectionGenerator::generate(
            PacketDirection::ServerBound,
            server_bound_packets,
        );
        let (server, _, _) = DirectionGenerator::generate(
            PacketDirection::ClientBound,
            client_bound_packets,
        );

        quote! {
            mod protocol {
                #client
                #server
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

        let play_packets = packets.split_off(&PacketIdentifier {
            direction: direction,
            stage: PacketStage::Play,
            id: 0.into(),
        });

        let login_packets = packets.split_off(&PacketIdentifier {
            direction: direction,
            stage: PacketStage::Login,
            id: 0.into(),
        });

        let status_packets = packets.split_off(&PacketIdentifier {
            direction: direction,
            stage: PacketStage::Status,
            id: 0.into(),
        });

        let handshaking_packets = packets.split_off(&PacketIdentifier {
            direction: direction,
            stage: PacketStage::Handshaking,
            id: 0.into(),
        });

        let (handshaking, handshaking_ident) = StageGenerator::generate(
            direction,
            PacketStage::Handshaking,
            handshaking_packets,
        );
        let (status, status_ident) = StageGenerator::generate(direction, PacketStage::Handshaking, login_packets);
        let (login, login_ident) = StageGenerator::generate(direction, PacketStage::Handshaking, status_packets);
        let (play, play_ident) = StageGenerator::generate(direction, PacketStage::Handshaking, play_packets);

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
            PacketDirection::ClientBound => Ident::new("client_bound", Span::call_site()),
            PacketDirection::ServerBound => Ident::new("server_bound", Span::call_site()),
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
        let stage_ident = Self::ident(stage);

        let (packet_tokens, packets_idents): (Vec<_>, Vec<_>) = packets
            .iter()
            .map(|(identifier, packet)| PacketGenerator::generate(identifier, packet))
            .unzip();

        let packet_names: Vec<_> = packets
            .iter()
            .map(|(_, p)| &p.name)
            .map(|name| Ident::new(name, Span::call_site()))
            .collect();

        let direction = match direction {
            PacketDirection::ClientBound => {
                quote! { feather_protocol::PacketDirection::ClientBound }
            }
            PacketDirection::ServerBound => {
                quote! { feather_protocol::PacketDirection::ServerBound }
            }
        };

        let stage = match stage {
            PacketStage::Handshaking => quote! { feather_protocol::PacketStage::Handshaking },
            PacketStage::Status => quote! { feather_protocol::PacketStage::Status },
            PacketStage::Login => quote! { feather_protocol::PacketStage::Login },
            PacketStage::Play => quote! { feather_protocol::PacketStage::Play },
        };

        (
            quote! {
                pub mod #stage_ident {
                    pub enum Packet {
                        #(#packet_names(#packets_idents))*
                    }

                    impl feather_protocol::Packet for #stage_ident {
                        type Direction = #direction;
                        type Stage = #stage;

                        fn id(&self) -> u64 {

                        }
                    }

                    #(#packet_tokens)*
                }
            },
            stage_ident,
        )
    }

    fn ident(stage: PacketStage) -> Ident {
        match stage {
            PacketStage::Handshaking => Ident::new("handshaking", Span::call_site()),
            PacketStage::Status => Ident::new("handshaking", Span::call_site()),
            PacketStage::Login => Ident::new("handshaking", Span::call_site()),
            PacketStage::Play => Ident::new("handshaking", Span::call_site()),
        }
    }
}

pub struct PacketGenerator;

impl PacketGenerator {
    pub fn generate(identifier: &PacketIdentifier, packet: &Packet) -> (TokenStream, Ident) {
        let ident = Ident::new(&format!("{}_packet", *packet.name), Span::call_site());
        let (custom_type, custom_type_ident) = CustomTypeGenerator::generate("packet", &packet.custom_type);

        let id_lit = LitInt::new(&identifier.id.to_string(), Span::call_site());
        let name_lit = LitStr::new(&*packet.name, Span::call_site());

        let direction = match identifier.direction {
            PacketDirection::ClientBound => {
                quote! { feather_protocol::PacketDirection::ClientBound }
            }
            PacketDirection::ServerBound => {
                quote! { feather_protocol::PacketDirection::ServerBound }
            }
        };

        let stage = match identifier.stage {
            PacketStage::Handshaking => quote! { feather_protocol::PacketStage::Handshaking },
            PacketStage::Status => quote! { feather_protocol::PacketStage::Status },
            PacketStage::Login => quote! { feather_protocol::PacketStage::Login },
            PacketStage::Play => quote! { feather_protocol::PacketStage::Play },
        };

        let serialization = PacketSerilizationGenerator::generate(&identifier, &packet);
        (
            quote! {
                pub mod #ident {
                    #custom_type

                    impl feather_protocol::Packet for #custom_type_ident {
                        type Direction = #direction;
                        type Stage = #stage;

                        fn id(&self) -> u64 {
                            #id_lit
                        }

                        fn name(&self) -> &'static str {
                            #name_lit
                        }
                    }

                    #serialization
                }
            },
            custom_type_ident,
        )
    }
}

pub struct PacketSerilizationGenerator;

impl PacketSerilizationGenerator {
    pub fn generate(identifier: &PacketIdentifier, packet: &Packet) -> TokenStream {
        quote! {}
    }

    fn generate_read(&self) -> TokenStream {
        quote! {}
    }
}

struct CustomTypeGenerator;

impl CustomTypeGenerator {
    pub fn generate(name: &str, custom_type: &CustomType) -> (TokenStream, Ident) {
        match custom_type {
            CustomType::Struct(fields) => Self::generate_struct(name, fields),
            CustomType::Enum { variant, variants } => Self::generate_enum(name, variants),
            CustomType::Unit => Self::generate_unit(name),
            CustomType::BitField(_) => (quote! {}, Self::ident(name)),
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
                (
                    Ident::new(&name.to_snake_case(), Span::call_site()),
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
                    #(#variant_idents),*
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
            Type::VarInt => (quote! {}, quote! { VarInt }),
            Type::Uuid => (quote! {}, quote! { uuid::Uuid }),
            Type::String(_) => (quote! {}, quote! { String }),
            Type::Nbt => (quote! {}, quote! { () }),
            Type::Array { schema, .. } => {
                let (tokens, ident) = TypeGenerator::generate(schema);
                (tokens, quote! { Vec<#ident> })
            }
            Type::Option(inner) => {
                let (tokens, ident) = TypeGenerator::generate(inner);
                (tokens, quote! { Option<#ident> })
            }
            Type::CustomType(name, custom_type) => {
                let (tokens, ident) = CustomTypeGenerator::generate(name, custom_type);
                (quote! { #tokens }, quote! { #ident })
            }
            Type::Key(_) => (quote! { () }, quote! {}),
        }
    }
}
