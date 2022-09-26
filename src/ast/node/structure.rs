use crate::parser::Span;

use super::{
    expression::Expr,
    identifier::Ident,
    type_signature::{Mutability, TypeSignature},
};

#[derive(Debug, Clone)]
pub struct Struct<'a> {
    pub name: Ident<'a>,
    pub attrs: Vec<StructAttr<'a>>,
}

#[derive(Debug, Clone)]
pub struct StructAttr<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub default_value: Option<Expr<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct StructInit<'a> {
    pub struct_name: Ident<'a>,
    pub values: Vec<StructInitValue<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct StructInitValue<'a> {
    pub name: Ident<'a>,
    pub value: Expr<'a>,
    pub span: Span<'a>,
}
