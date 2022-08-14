use crate::{ir::context::IrCtx, symbols::symbol_table::symbol_table_zipper::SymbolTableZipper};

use super::{
    type_signature::{BuiltinType, TypeEvalError, TypeSignature, Typed},
    NodeRef,
};

#[derive(Debug, Clone)]
pub struct EscapeBlock<'a> {
    pub content: &'a str,
    pub type_sig: Option<TypeSignature<'a>>,
}

impl<'a> Typed<'a> for NodeRef<'a, EscapeBlock<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        if let Some(sig) = ctx[*self].type_sig {
            Ok(sig)
        } else {
            Ok(ctx.get_builtin_type_sig(BuiltinType::Untyped))
        }
    }

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        ctx[*self].type_sig
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        ctx[*self].type_sig = Some(new_type);
        Ok(())
    }
}
