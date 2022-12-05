use crate::{
    error_message::error_formatter::Spanned,
    ir::{ast_lowering::IrLowerable, context::IrCtx, late_init::LateInit},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    type_signature::{TypeEvalError, TypeSignature, TypeSignatureParent, Typed},
    IrAlloc, NodeRef,
};

#[derive(Debug, Clone)]
pub struct EscapeBlock<'a> {
    pub content: &'a str,
    pub type_sig: LateInit<TypeSignature<'a>>,
    pub span: Span<'a>,
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

impl<'a> Spanned<'a> for NodeRef<'a, EscapeBlock<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::escape_block::EscapeBlock<'a> {
    type IrType = EscapeBlock<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let esc_blk = EscapeBlock {
            content: self.content,
            type_sig: LateInit::empty(),
            span: self.span,
        }
        .allocate(ctx);

        ctx[esc_blk].type_sig = self
            .type_sig
            .map(|t| t.into_ir_type(ctx, TypeSignatureParent::EscapeBlock(esc_blk)))
            .unwrap_or_else(|| ctx.make_type_var(TypeSignatureParent::EscapeBlock(esc_blk)))
            .into();

        esc_blk
    }
}
