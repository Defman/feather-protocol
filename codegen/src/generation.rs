use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::LitStr;

use indexmap::IndexMap;
use std::collections::BTreeMap;

use feather_protocol_spec::{
    ArrayLength, BitField, CustomType, FieldName, FieldType, FlagName, FloatType, IntegerType,
    Literal, PacketDirection, PacketIdentifier, PacketStage, PrimitiveType, Protocol,
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
            .map(CustomTypeGenerator::generate)
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
        mut packets: BTreeMap<PacketIdentifier, CustomType>,
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

        let (handshaking_ident, handshaking_tokens) =
            StageGenerator::generate(direction, PacketStage::Handshaking, handshaking_packets);
        let (status_ident, status_tokens) =
            StageGenerator::generate(direction, PacketStage::Status, login_packets);
        let (login_ident, login_tokens) =
            StageGenerator::generate(direction, PacketStage::Login, status_packets);
        let (play_ident, play_tokens) =
            StageGenerator::generate(direction, PacketStage::Play, play_packets);

        (
            quote! {
                pub mod #direction_ident {
                    #handshaking_tokens
                    #status_tokens
                    #login_tokens
                    #play_tokens
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
        _direction: PacketDirection,
        stage: PacketStage,
        packets: BTreeMap<PacketIdentifier, CustomType>,
    ) -> (Ident, TokenStream) {
        let stage_ident = Self::ident(stage);

        let (packet_idents, packet_tokens): (Vec<_>, Vec<_>) =
            packets.iter().map(PacketGenerator::generate).unzip();

        let _packet_ids: Vec<_> = packets.iter().map(|(i, _)| *i.id()).collect();

        let tokens = quote! {
            pub mod #stage_ident {
                pub enum Packet {
                    #(#packet_idents(#packet_idents)),*
                }

                #(
                    impl From<#packet_idents> for Packet {
                        fn from(packet: #packet_idents) -> Self {
                            Packet::#packet_idents(packet)
                        }
                    }
                )*

                #(#packet_tokens)*
            }
        };

        (stage_ident, tokens)
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
    fn generate(packet: (&PacketIdentifier, &CustomType)) -> (Ident, TokenStream) {
        let custom_type = packet.1;
        let (custom_type_ident, custom_type_tokens) = CustomTypeGenerator::generate(custom_type);

        let packet_ident = Self::ident(custom_type.name());

        let tokens = quote! {
            pub use #packet_ident::#custom_type_ident;
            mod #packet_ident {
                #custom_type_tokens
            }
        };
        (custom_type_ident, tokens)
    }

    fn generate_write(packet: (&PacketIdentifier, &CustomType)) -> TokenStream {
        let custom_type = packet.1;
        let ident = Self::ident(custom_type.name());
        let custom_type_write = CustomTypeGenerator::generate_write(custom_type);
        quote! {
            impl TryWriteInto for #ident {
                fn write(&self, buf: &mut impl BufMut) -> usize {
                    let mut total = 0;
                    total += buf.write(<Self as Packet>::ID);
                }
            }
        }
    }

    fn ident(name: &str) -> Ident {
        use heck::SnakeCase;
        let name = name.to_snake_case();
        Ident::new(&name, Span::call_site())
    }
}

pub struct CustomTypeGenerator;

impl CustomTypeGenerator {
    fn generate(custom_type: &CustomType) -> (Ident, TokenStream) {
        match custom_type {
            CustomType::Enum {
                name,
                variant,
                variants,
            } => Self::generate_enum(name, variant, variants),
            CustomType::Struct { name, fields } => Self::generate_struct(name, fields),
            CustomType::BitField { name, fields } => Self::generate_bit_field(name, fields),
            CustomType::BitFlags {
                name,
                field_type,
                flags,
            } => Self::generate_bit_flags(name, field_type, flags),
            CustomType::Unit { name } => Self::generate_unit(name),
        }
    }

    fn generate_enum(
        name: &str,
        variant: &FieldType,
        variants: &BTreeMap<Literal, CustomType>,
    ) -> (Ident, TokenStream) {
        let ident = Self::ident(name);

        let (variants_idents, variants_tokens): (Vec<_>, Vec<_>) =
            variants.values().map(Self::generate).unzip();

        let (variant_ident, variant_tokens) = FieldGenerator::tokenize_field_type(variant);

        let tokens = quote! {
            pub enum #ident {
                #(#variants_idents(#variants_idents)),*,
                Other(#variant_ident),
            }

            #(#variants_tokens)*
            #(#variant_tokens)
        };
        (ident, tokens)
    }

    fn generate_struct(
        name: &str,
        fields: &IndexMap<FieldName, FieldType>,
    ) -> (Ident, TokenStream) {
        let ident = Self::ident(name);

        let (field_idents, field_types): (Vec<_>, Vec<_>) =
            fields.iter().map(FieldGenerator::generate).unzip();

        let (field_types, field_type_custom_types): (Vec<_>, Vec<_>) =
            field_types.into_iter().unzip();

        let tokens = quote! {
            pub struct #ident {
                #(#field_idents: #field_types),*
            }

            #(#field_type_custom_types)*
        };

        (ident, tokens)
    }

    fn generate_bit_field(
        name: &str,
        _fields: &IndexMap<FieldName, BitField>,
    ) -> (Ident, TokenStream) {
        let ident = Self::ident(name);

        (ident, quote! {})
    }

    fn generate_bit_flags(
        name: &str,
        integer: &IntegerType,
        flags: &BTreeMap<u64, FlagName>,
    ) -> (Ident, TokenStream) {
        let ident = Self::ident(name);

        let field_type = FieldGenerator::tokenize_integer(integer);
        let (flag_bits, flag_idents): (Vec<_>, Vec<_>) = flags
            .iter()
            .map(|(bits, name)| {
                (
                    quote! { #bits },
                    Ident::new(&name.to_uppercase(), Span::call_site()),
                )
            })
            .unzip();

        let tokens = quote! {
            bitflags::bitflags! {
                pub struct #ident: #field_type {
                    #(const #flag_idents = #flag_bits;)*
                }
            }
        };

        (ident, tokens)
    }

    fn generate_unit(name: &str) -> (Ident, TokenStream) {
        let ident = Self::ident(name);

        let tokens = quote! {
            pub struct #ident;
        };
        (ident, tokens)
    }

    fn generate_write(custom_type: &CustomType) -> TokenStream {
        let ident = Self::ident(custom_type.name());
        
        match custom_type {
            CustomType::Struct {
                fields,
                ..
            } => {
                let field_names: Vec<_> = fields.keys().map(|name| FieldGenerator::ident(&name)).collect();
                quote! {
                    impl TryWriteInto for #ident {
                        fn write(&self, buf: &mut impl BufMut) -> usize {
                            #(self.#field_names.write(buf);)*
                        }
                    }
                }
            },
            CustomType::Enum {
                variant,
                variants,
                ..
            } => {
                // let (literals, idents) = variants
                //     .iter()
                //     .map(|(literal, custom_type)| )

                quote! {
                    // impl TryWriteInto for #ident {
                    //     fn write(&self, buf: &mut impl BufMut) -> usize {
                    //         match self {
                    //             #(
                    //                 #name::#idents(inner) => inner.write(buf)
                    //             ),*
                    //         }
                    //     }
                    // }
                }
            },
            CustomType::Unit { .. } => quote! {
                impl TryWriteInto for #ident {
                    fn write(&self, buf: &mut impl BufMut) -> usize {
                        0
                    }
                }
            },
            CustomType::BitField { .. } => quote! {
                impl TryWriteInto for #ident {
                    fn write(&self, buf: &mut impl BufMut) -> usize {
                        0
                    }
                }
            },
            CustomType::BitFlags { .. } => quote! {
                impl TryWriteInto for #ident {
                    fn write(&self, buf: &mut impl BufMut) -> usize {
                        0
                    }
                }
            }
        }
    }

    fn ident(name: &str) -> Ident {
        use heck::CamelCase;
        let name = name.to_camel_case();
        Ident::new(&name, Span::call_site())
    }
}

pub struct FieldGenerator;
impl FieldGenerator {
    fn generate(field: (&FieldName, &FieldType)) -> (Ident, (TokenStream, TokenStream)) {
        let field_name = field.0;
        let ident = Self::ident(&field_name);

        let field_type = field.1;

        let field_type_tokens = Self::tokenize_field_type(field_type);
        (ident, field_type_tokens)
    }

    fn tokenize_field_type(field_type: &FieldType) -> (TokenStream, TokenStream) {
        match field_type {
            FieldType::Primitive(primitive_type) => Self::tokenize_primitive(primitive_type),
            FieldType::Integer(integer_type) => (Self::tokenize_integer(integer_type), quote! {}),
            FieldType::Float(float_type) => (Self::tokenize_float(float_type), quote! {}),
            FieldType::CustomType(custom_type) => {
                let (ident, tokens) = CustomTypeGenerator::generate(custom_type);
                (ident.to_token_stream(), tokens)
            }
        }
    }

    fn tokenize_primitive(primitive_type: &PrimitiveType) -> (TokenStream, TokenStream) {
        match primitive_type {
            PrimitiveType::Boolean => (quote! { bool }, quote! {}),
            PrimitiveType::Uuid => (quote! { uuid::Uuid }, quote! {}),
            PrimitiveType::String(_) => (quote! { String }, quote! {}),
            PrimitiveType::Nbt => (quote! { nbt::Blob }, quote! {}),
            PrimitiveType::Array { field_type, .. } => {
                let (ident, tokens) = Self::tokenize_field_type(field_type);
                (quote! {Vec<#ident>}, tokens)
            }
            PrimitiveType::Option(inner) => {
                let (inner_ident, inner_tokens) = Self::tokenize_field_type(inner);
                (quote! { Option<#inner_ident> }, inner_tokens)
            }
            PrimitiveType::Shared(name) => (
                CustomTypeGenerator::ident(name).to_token_stream(),
                quote! {},
            ),
        }
    }

    fn tokenize_integer(integer_type: &IntegerType) -> TokenStream {
        match integer_type {
            IntegerType::U8 => quote! { u8 },
            IntegerType::I8 => quote! { i8 },
            IntegerType::U16 => quote! { u16 },
            IntegerType::I16 => quote! { i16 },
            IntegerType::U32 => quote! { u32 },
            IntegerType::I32 | IntegerType::VarInt => quote! { i32 },
            IntegerType::U64 => quote! { u64 },
            IntegerType::I64 | IntegerType::VarLong => quote! { i64 },
        }
    }

    fn tokenize_float(float_type: &FloatType) -> TokenStream {
        match float_type {
            FloatType::F32 => quote! { f32 },
            FloatType::F64 => quote! { f64 },
        }
    }

    fn ident(name: &str) -> Ident {
        use heck::SnakeCase;
        let name = name.to_snake_case();
        Ident::new(&name, Span::call_site())
    }
}
