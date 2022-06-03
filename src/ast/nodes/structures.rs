use super::{
    expressions::Expr,
    identifier::Ident,
    type_signature::{Mutability, TypeSignature},
};

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: Ident<'a>,
    pub attrs: Vec<StructAttr<'a>>,
}

#[derive(Debug)]
pub struct StructAttr<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub default_value: Option<Expr<'a>>,
}
