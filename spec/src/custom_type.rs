use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::primitives::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomType {
    Enum {
        name: CustomTypeName,
        variant: EnumVariant,
        variants: BTreeMap<Literal, CustomType>,
    },
    Struct {
        name: CustomTypeName,
        fields: IndexMap<FieldName, Type>
    },
    BitField {
        name: CustomTypeName,
        fields: IndexMap<FieldName, BitField>,
    },
    BitFlags {
        name: CustomTypeName,
        #[serde(rename = "type")]
        kind: Box<Type>,
        flags: BTreeMap<u64, String>,
    },
    Unit {
        name: CustomTypeName,
    },
}

impl CustomType {
    pub fn name(&self) -> &str {
        match self {
            CustomType::Enum { name, ..} => name,
            CustomType::Struct { name, ..} => name,
            CustomType::BitField { name, ..} => name,
            CustomType::BitFlags { name, ..} => name,
            CustomType::Unit { name, ..} => name,
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
pub enum EnumVariant {
    Key(String),
    Prefixed(Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrayLength {
    RemainingLength,
    FixedLength(u64),
    Prefixed(Box<Type>),
    Key(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Primitive {
    Boolean,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    VarInt,
    VarLong,
    Uuid,
    String(#[serde(default)] u16),
    Nbt,
    Array {
        length: ArrayLength,
        #[serde(rename = "type")]
        kind: Box<Type>,
    },
    None,
    Option(Box<Type>),
    Key(Box<Type>),
    Shared(CustomTypeName),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Type {
    Primitive(Primitive),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Literal {
    String(String),
    Boolean(bool),
    Int(i64),
}