use crate::{
    ir::{context::IrCtx, late_init::LateInit},
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    expression::Expr,
    identifier::Ident,
    type_signature::{TypeEvalError, TypeSignature, Typed},
    NodeRef,
};

#[derive(Debug, Clone)]
pub struct UnresolvedMemberAccess<'a> {
    pub object: Option<NodeRef<'a, Expr<'a>>>,
    pub member_name: LateInit<Ident<'a>>,
    pub items: Vec<NodeRef<'a, Expr<'a>>>,
    pub type_sig: TypeSignature<'a>,
}

impl<'a> Typed<'a> for NodeRef<'a, UnresolvedMemberAccess<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok(ctx[*self].type_sig)
    }

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        Some(ctx[*self].type_sig)
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        ctx[*self].type_sig = new_type;
        Ok(())
    }
}
