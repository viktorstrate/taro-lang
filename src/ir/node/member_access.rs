use crate::{
    error_message::error_formatter::Spanned,
    ir::{ast_lowering::IrLowerable, context::IrCtx, late_init::LateInit},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    expression::Expr,
    identifier::{Ident, IdentParent},
    type_signature::{TypeEvalError, TypeSignature, TypeSignatureParent, Typed},
    IrAlloc, NodeRef,
};

#[derive(Debug, Clone)]
pub struct UnresolvedMemberAccess<'a> {
    pub object: Option<NodeRef<'a, Expr<'a>>>,
    pub member_name: LateInit<Ident<'a>>,
    pub items: Option<(Vec<NodeRef<'a, Expr<'a>>>, Span<'a>)>,
    pub type_sig: LateInit<TypeSignature<'a>>,
    pub span: Span<'a>,
}

impl<'a> Spanned<'a> for NodeRef<'a, UnresolvedMemberAccess<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, UnresolvedMemberAccess<'a>> {
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

impl<'a> IrLowerable<'a> for crate::ast::node::member_access::MemberAccess<'a> {
    type IrType = UnresolvedMemberAccess<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let object = self.object.map(|obj| obj.ir_lower(ctx));
        let items = self.items.map(|(span, items)| {
            (
                items.into_iter().map(|item| item.ir_lower(ctx)).collect(),
                span,
            )
        });

        let mem_acc = UnresolvedMemberAccess {
            object,
            member_name: LateInit::empty(),
            items,
            type_sig: LateInit::empty(),
            span: self.span,
        }
        .allocate(ctx);

        ctx[mem_acc].member_name = ctx
            .make_unresolved_ident(
                self.member_name,
                IdentParent::MemberAccessMemberName(mem_acc).into(),
            )
            .into();

        ctx[mem_acc].type_sig = ctx
            .make_type_var(TypeSignatureParent::MemberAccess(mem_acc))
            .into();

        mem_acc
    }
}
