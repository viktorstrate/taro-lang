use super::{identifier::Ident, type_signature::TypeSignature};

#[derive(Debug, Clone, PartialEq)]
pub struct Enum<'a> {
    pub name: Ident<'a>,
    pub values: Vec<EnumValue<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue<'a> {
    pub name: Ident<'a>,
    pub items: Vec<TypeSignature<'a>>,
}
