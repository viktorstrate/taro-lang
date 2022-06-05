use crate::{ast::ref_generator::RefID, symbols::symbol_table::SymbolValue};

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

#[derive(Debug, Clone)]
pub struct StructInit<'a> {
    pub name: Ident<'a>,
    pub values: Vec<StructInitValue<'a>>,
}

#[derive(Debug, Clone)]
pub struct StructInitValue<'a> {
    pub name: Ident<'a>,
    pub value: Expr<'a>,
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

impl<'a> Typed<'a> for StructInit<'a> {
    type Error = ExprValueError<'a>;

    fn type_sig(
        &self,
        symbols: &mut crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        let st = symbols
            .locate(&self.name)
            .expect("struct init base declaration should exist");

        let st = match st {
            SymbolValue::StructDecl(st) => st,
            _ => unreachable!(),
        };

        Ok(TypeSignature::Struct {
            name: st.name.clone(),
            ref_id: st.ref_id,
        })
    }
}
