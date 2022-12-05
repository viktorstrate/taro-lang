use crate::{
    ir::{
        ast_lowering::IrLowerable,
        context::IrCtx,
        late_init::LateInit,
        node::{identifier::IdentParent, type_signature::TypeSignatureParent, IrAlloc},
    },
    parser::Span,
};

use super::{
    function::FunctionArg,
    identifier::{Ident, Identifiable},
    type_signature::TypeSignature,
    NodeRef,
};

#[derive(Debug, Clone)]
pub struct Trait<'a> {
    pub name: LateInit<Ident<'a>>,
    pub attrs: Vec<NodeRef<'a, TraitFuncAttr<'a>>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct TraitFuncAttr<'a> {
    pub name: LateInit<Ident<'a>>,
    pub args: Vec<NodeRef<'a, FunctionArg<'a>>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub span: Span<'a>,
}

impl<'a> Identifiable<'a> for Trait<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.name
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::traits::Trait<'a> {
    type IrType = Trait<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let tr = Trait {
            name: LateInit::empty(),
            attrs: self
                .attrs
                .into_iter()
                .map(|attr| attr.ir_lower(ctx))
                .collect(),
            span: self.span,
        }
        .allocate(ctx);

        ctx[tr].name = ctx.make_ident(self.name, IdentParent::TraitName(tr)).into();

        tr
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::traits::TraitFuncAttr<'a> {
    type IrType = TraitFuncAttr<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let f = TraitFuncAttr {
            name: LateInit::empty(),
            args: Vec::new(),
            return_type: None,
            span: self.span,
        }
        .allocate(ctx);

        ctx[f].name = ctx
            .make_ident(self.name, IdentParent::TraitFuncAttrName(f))
            .into();

        ctx[f].return_type = self
            .return_type
            .map(|t| t.into_ir_type(ctx, TypeSignatureParent::TraitFuncAttr(f)));

        ctx[f].args = self.args.into_iter().map(|arg| arg.ir_lower(ctx)).collect();

        f
    }
}
