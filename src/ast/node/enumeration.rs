use super::{expression::Expr, identifier::Ident, type_signature::TypeSignature};

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

#[derive(Debug, Clone)]
pub struct EnumInit<'a> {
    pub enum_name: Option<Ident<'a>>,
    pub enum_value: Ident<'a>,
    pub items: Vec<Expr<'a>>,
}
