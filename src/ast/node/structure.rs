use crate::ast::ref_generator::RefID;

use super::{
    expression::{Expr, ExprValueError},
    identifier::{Ident, Identifiable},
    type_signature::{Mutability, TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub struct Struct<'a> {
    pub name: Ident<'a>,
    pub attrs: Vec<StructAttr<'a>>,
    pub ref_id: RefID,
}

#[derive(Debug, Clone)]
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

impl<'a> Typed<'a> for Struct<'a> {
    type Error = ExprValueError<'a>;

    fn type_sig(
        &self,
        _symbols: &mut crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        Ok(TypeSignature::Struct {
            name: self.name.clone(),
            ref_id: self.ref_id,
        })
    }
}

impl<'a> Typed<'a> for StructAttr<'a> {
    type Error = ExprValueError<'a>;

    fn type_sig(
        &self,
        symbols: &mut crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        match &self.default_value {
            Some(value) => value.type_sig(symbols),
            None => Ok(self
                .type_sig
                .clone()
                .expect("struct should have at least a type signature or a default value")),
        }
    }
}
