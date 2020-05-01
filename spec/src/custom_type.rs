use crate::primitives::*;
use indexmap::IndexMap;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomType {
    Enum {
        name: CustomTypeName,
        variant: Box<FieldType>,
        variants: BTreeMap<Literal, CustomType>,
    },
    Struct {
        name: CustomTypeName,
        fields: IndexMap<FieldName, FieldType>,
    },
    BitField {
        name: CustomTypeName,
        fields: IndexMap<FieldName, BitField>,
    },
    BitFlags {
        name: CustomTypeName,
        field_type: Box<IntegerType>,
        flags: BTreeMap<u64, FlagName>,
    },
    Unit {
        name: CustomTypeName,
    },
}

impl CustomType {
    pub fn name(&self) -> &str {
        match self {
            CustomType::Enum { name, .. } => name,
            CustomType::Struct { name, .. } => name,
            CustomType::BitField { name, .. } => name,
            CustomType::BitFlags { name, .. } => name,
            CustomType::Unit { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitField {
    Boolean,
    I8(u8),
    U8(u8),
    I16(u8),
    U16(u8),
    I32(u8),
    U32(u8),
    I64(u8),
    U64(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrayLength {
    RemainingLength,
    FixedLength(u64),
    Prefixed(Box<IntegerType>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegerType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    VarInt,
    VarLong,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloatType {
    F32,
    F64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Boolean,
    Uuid,
    String(#[serde(default)] u16),
    Nbt,
    Array {
        length: ArrayLength,
        field_type: Box<FieldType>,
    },
    Option(Box<FieldType>),
    Shared(CustomTypeName),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum FieldType {
    Primitive(PrimitiveType),
    Float(FloatType),
    Integer(IntegerType),
    CustomType(CustomType),
}

impl<'de> Deserialize<'de> for FieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {

        #[derive(Deserialize)]
        enum Field {
            U8,
            I8,
            U16,
            I16,
            U32,
            I32,
            U64,
            I64,
            VarInt,
            VarLong,
            F32,
            F64,
            Boolean,
            Uuid,
            String(u16),
            Nbt,
            Array {
                length: ArrayLength,
                field_type: Box<FieldType>,
            },
            Option(Box<FieldType>),
            Shared(CustomTypeName),
            Enum {
                name: CustomTypeName,
                variant: Box<FieldType>,
                variants: BTreeMap<Literal, CustomType>,
            },
            Struct {
                name: CustomTypeName,
                fields: IndexMap<FieldName, FieldType>,
            },
            BitField {
                name: CustomTypeName,
                fields: IndexMap<FieldName, BitField>,
            },
            BitFlags {
                name: CustomTypeName,
                field_type: Box<IntegerType>,
                flags: BTreeMap<u64, FlagName>,
            },
            Unit {
                name: CustomTypeName,
            },
        }

        let field = <Field as Deserialize>::deserialize(deserializer)?;
        Ok(match field {
            Field::U8 => FieldType::Integer(IntegerType::U8),
            Field::I8 => FieldType::Integer(IntegerType::I8),
            Field::U16 => FieldType::Integer(IntegerType::U16),
            Field::I16 => FieldType::Integer(IntegerType::I16),
            Field::U32 => FieldType::Integer(IntegerType::U32),
            Field::I32 => FieldType::Integer(IntegerType::I32),
            Field::U64 => FieldType::Integer(IntegerType::U64),
            Field::I64 => FieldType::Integer(IntegerType::I64),
            Field::VarInt => FieldType::Integer(IntegerType::VarInt),
            Field::VarLong => FieldType::Integer(IntegerType::VarLong),
            Field::F32 => FieldType::Float(FloatType::F32),
            Field::F64 => FieldType::Float(FloatType::F64),
            Field::Boolean => FieldType::Primitive(PrimitiveType::Boolean),
            Field::Uuid => FieldType::Primitive(PrimitiveType::Uuid),
            Field::String(n) => FieldType::Primitive(PrimitiveType::String(n)),
            Field::Nbt => FieldType::Primitive(PrimitiveType::Nbt),
            Field::Array {
                length,
                field_type,
            } => FieldType::Primitive(PrimitiveType::Array {
                length,
                field_type,
            }),
            Field::Option(inner) => FieldType::Primitive(PrimitiveType::Option(inner)),
            Field::Shared(name) => FieldType::Primitive(PrimitiveType::Shared(name)),
            Field::Enum {
                name,
                variant,
                variants,
            } => FieldType::CustomType(CustomType::Enum {
                name,
                variant,
                variants,
            }),
            Field::Struct {
                name,
                fields,
            } => FieldType::CustomType(CustomType::Struct {
                name,
                fields
            }),
            Field::BitField {
                name,
                fields,
            } => FieldType::CustomType(CustomType::BitField {
                name,
                fields
            }),
            Field::BitFlags {
                name,
                field_type,
                flags,
            } => FieldType::CustomType(CustomType::BitFlags {
                name,
                field_type,
                flags
            }),
            Field::Unit {
                name,
            } => FieldType::CustomType(CustomType::Unit {
                name
            }),
        })
    }
}

impl From<IntegerType> for FieldType {
    fn from(integer_type: IntegerType) -> FieldType {
        FieldType::Integer(integer_type)
    }
}

impl From<FloatType> for FieldType {
    fn from(float_type: FloatType) -> FieldType {
        FieldType::Float(float_type)
    }
}

impl From<PrimitiveType> for FieldType {
    fn from(primitive_type: PrimitiveType) -> FieldType {
        FieldType::Primitive(primitive_type)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Literal {
    String(String),
    Boolean(bool),
    Int(i64),
}
