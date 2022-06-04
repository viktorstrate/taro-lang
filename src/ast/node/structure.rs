use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
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

impl<'a> Identifiable<'a> for Struct<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

impl<'a> Identifiable<'a> for StructAttr<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}
