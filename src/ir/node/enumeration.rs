use crate::{ir::context::IrCtx, symbols::symbol_table::symbol_table_zipper::SymbolTableZipper};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    type_signature::{TypeEvalError, TypeSignature, TypeSignatureValue, Typed},
    NodeRef,
};

#[derive(Debug, PartialEq)]
pub struct Enum<'a> {
    pub name: Ident<'a>,
    pub values: Vec<NodeRef<'a, EnumValue<'a>>>,
    pub type_sig: TypeSignature<'a>,
}

#[derive(Debug, PartialEq)]
pub struct EnumValue<'a> {
    pub name: Ident<'a>,
    pub items: Vec<TypeSignature<'a>>,
}

pub struct EnumInit<'a> {
    pub enum_name: Option<Ident<'a>>,
    pub enum_value: Ident<'a>,
    pub items: Vec<NodeRef<'a, Expr<'a>>>,
}

impl<'a> Identifiable<'a> for Enum<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

impl<'a> Identifiable<'a> for EnumValue<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

impl<'a> Typed<'a> for NodeRef<'a, Enum<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok(ctx.nodes.enms[self.id].type_sig)
    }
}

impl<'a> Typed<'a> for NodeRef<'a, EnumValue<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok(self.specified_type(ctx).unwrap())
    }

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        let items = ctx.nodes[*self].items.clone();
        Some(ctx.get_type_sig(TypeSignatureValue::Tuple(items)))
    }

    fn specify_type(
        &mut self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        let TypeSignatureValue::Tuple(tuple) = &ctx[new_type] else {
            unreachable!("specified type expected to be tuple");
        };

        assert_eq!(tuple.len(), ctx[*self].items.len());

        ctx[*self].items = tuple.clone();
        Ok(())
    }
}
