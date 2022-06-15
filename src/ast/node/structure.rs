use crate::{ast::ref_generator::RefID, symbols::symbol_table::SymbolValue};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    type_signature::{Mutability, TypeEvalError, TypeSignature, Typed},
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
    fn eval_type(
        &self,
        _symbols: &mut crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok(TypeSignature::Struct {
            name: self.name.clone(),
            ref_id: self.ref_id,
        })
    }
}

impl<'a> Typed<'a> for StructAttr<'a> {
    fn eval_type(
        &self,
        symbols: &mut crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match &self.default_value {
            Some(value) => value.eval_type(symbols),
            None => Ok(self
                .type_sig
                .clone()
                .expect("struct should have at least a type signature or a default value")),
        }
    }

    fn specified_type(&self) -> Option<&TypeSignature<'a>> {
        self.type_sig.as_ref()
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) {
        self.type_sig = Some(new_type);
    }
}

impl<'a> Typed<'a> for StructInit<'a> {
    fn eval_type(
        &self,
        symbols: &mut crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let st = symbols
            .lookup(&self.name)
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
