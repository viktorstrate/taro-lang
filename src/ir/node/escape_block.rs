use crate::{
    ir::{context::IrCtx, late_init::LateInit},
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    type_signature::{TypeEvalError, TypeSignature, Typed},
    NodeRef,
};

#[derive(Debug, Clone)]
pub struct EscapeBlock<'a> {
    pub content: &'a str,
    pub type_sig: LateInit<TypeSignature<'a>>,
}

impl<'a> Typed<'a> for NodeRef<'a, EscapeBlock<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok((*ctx[*self].type_sig).clone())
    }

    fn specified_type(&self, ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        Some((*ctx[*self].type_sig).clone())
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        ctx[*self].type_sig = new_type.into();
        Ok(())
    }
}
