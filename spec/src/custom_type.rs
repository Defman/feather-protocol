use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::cmp::Ordering;
use crate::primitives::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomType {
    Enum {
        variant: EnumVariant,
        variants: BTreeMap<VariantKey, CustomType>,
    },
    Struct(IndexMap<FieldName, Type>),
    BitField(IndexMap<String, BitField>),
    BitFlags {
        #[serde(rename = "type")]
        kind: Box<Type>,
        flags: BTreeMap<u64, String>,
    },
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variant {
    pub name: VariantName,
    pub custom_type: CustomType,
}

#[derive(Debug, Clone, Eq, Ord, Serialize, Deserialize)]
pub struct VariantKey(pub Literal, pub VariantName);

impl PartialEq<Self> for VariantKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialOrd<Self> for VariantKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
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
pub enum Type {
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
        #[serde(rename = "type")]
        kind: Box<Type>,
        length: ArrayLength,
    },
    Option(Box<Type>),
    CustomType(String, CustomType),
    Key(Box<Type>),
    Constant(Literal),
    Shared(SharedTypeId),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Literal {
    String(String),
    Boolean(bool),
    Int(i64),
}