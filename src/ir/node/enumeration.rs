use crate::{
    ir::{context::IrCtx, late_init::LateInit},
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    expression::Expr,
    identifier::{Ident, IdentKey, Identifiable},
    type_signature::{
        TypeEvalError, TypeSignature, TypeSignatureContext, TypeSignatureParent,
        TypeSignatureValue, Typed,
    },
    NodeRef,
};

#[derive(Debug, Clone)]
pub struct Enum<'a> {
    pub name: LateInit<Ident<'a>>,
    pub values: Vec<NodeRef<'a, EnumValue<'a>>>,
    pub type_sig: LateInit<TypeSignature<'a>>,
}

#[derive(Debug, Clone)]
pub struct EnumValue<'a> {
    pub name: LateInit<Ident<'a>>,
    pub items: LateInit<Vec<TypeSignature<'a>>>,
}

impl<'a> NodeRef<'a, Enum<'a>> {
    pub fn lookup_value(
        self,
        ctx: &IrCtx<'a>,
        ident: Ident<'a>,
    ) -> Option<(usize, NodeRef<'a, EnumValue<'a>>)> {
        ctx[self]
            .values
            .iter()
            .enumerate()
            .find(|(_, val)| IdentKey::idents_eq(ctx, *ctx[**val].name, ident))
            .map(|(i, val)| (i, *val))
    }
}

#[derive(Debug, Clone)]
pub struct EnumInit<'a> {
    pub enum_name: Ident<'a>,
    pub enum_value: Ident<'a>,
    pub items: Vec<NodeRef<'a, Expr<'a>>>,
}

impl<'a> NodeRef<'a, EnumInit<'a>> {
    pub fn lookup_enum(
        &self,
        ctx: &IrCtx<'a>,
        symbols: &SymbolTableZipper<'a>,
    ) -> Option<NodeRef<'a, Enum<'a>>> {
        symbols
            .lookup(ctx, ctx[*self].enum_name)
            .map(|sym| sym.unwrap_enum(ctx))
    }
}

impl<'a> Identifiable<'a> for Enum<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.name
    }
}

impl<'a> Identifiable<'a> for EnumValue<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.name
    }
}

impl<'a> Typed<'a> for NodeRef<'a, Enum<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok((*ctx.nodes.enms[self.id].type_sig).clone())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, EnumValue<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        // Ok(self.specified_type(ctx).unwrap())
        let items = (*ctx[*self].items).clone();

        Ok(ctx.get_type_sig(
            TypeSignatureValue::Tuple(items.into()),
            TypeSignatureContext {
                parent: TypeSignatureParent::EnumValue(*self),
                type_span: None,
            }
            .alloc(),
        ))
    }

    fn specified_type(&self, _ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        None
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        let TypeSignatureValue::Tuple(tuple) = &ctx[&new_type] else {
            unreachable!("specified type expected to be tuple");
        };

        assert_eq!(
            tuple.len(),
            ctx[*self].items.len(),
            "enum value length match"
        );

        ctx[*self].items = tuple.clone().into();
        Ok(())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, EnumInit<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let enm_name = ctx[*self].enum_name;

        Ok(ctx.get_type_sig(
            TypeSignatureValue::Enum { name: enm_name },
            TypeSignatureContext {
                parent: TypeSignatureParent::EnumInit(*self),
                type_span: None,
            }
            .alloc(),
        ))
    }
}
