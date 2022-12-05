use crate::{
    error_message::error_formatter::Spanned,
    ir::{ast_lowering::IrLowerable, context::IrCtx, late_init::LateInit},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    identifier::{Ident, IdentParent, Identifiable},
    type_signature::{TypeEvalError, TypeSignature, TypeSignatureParent, Typed},
    IrAlloc, NodeRef,
};

#[derive(Debug, Clone)]
pub struct ExternalObject<'a> {
    pub ident: LateInit<Ident<'a>>,
    pub type_sig: LateInit<TypeSignature<'a>>,
    pub span: Span<'a>,
}

impl<'a> Typed<'a> for NodeRef<'a, ExternalObject<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok(self.specified_type(ctx).unwrap())
    }

    fn specified_type(&self, ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        Some((*ctx[*self].type_sig).clone().into())
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        assert_eq!(
            *ctx[*self].type_sig, new_type,
            "tried to change the type of an `external object`"
        );
        Ok(())
    }
}

impl<'a> Spanned<'a> for NodeRef<'a, ExternalObject<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> Identifiable<'a> for ExternalObject<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.ident
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::external::ExternalObject<'a> {
    type IrType = ExternalObject<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let obj = ExternalObject {
            ident: LateInit::empty(),
            type_sig: LateInit::empty(),
            span: self.span,
        }
        .allocate(ctx);

        ctx[obj].ident = ctx
            .make_ident(self.ident, IdentParent::ExternObjName(obj))
            .into();

        ctx[obj].type_sig = self
            .type_sig
            .into_ir_type(ctx, TypeSignatureParent::ExternObjType(obj))
            .into();

        obj
    }
}
