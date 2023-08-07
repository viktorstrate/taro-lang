use crate::{
    error_message::error_formatter::Spanned,
    ir::{
        ast_lowering::IrLowerable,
        context::IrCtx,
        late_init::LateInit,
        node::{
            identifier::IdentParent,
            type_signature::{TypeSignatureContext, TypeSignatureParent, TypeSignatureValue},
        },
    },
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    identifier::{Ident, Identifiable},
    type_signature::{TypeEvalError, TypeSignature, Typed},
    IrAlloc, NodeRef,
};

#[derive(Debug, Clone)]
pub struct GenericsDecl<'a> {
    pub generics: Vec<NodeRef<'a, GenericType<'a>>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct GenericType<'a> {
    pub name: LateInit<Ident<'a>>,
    pub span: Span<'a>,
}

impl<'a> Spanned<'a> for NodeRef<'a, GenericsDecl<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> Spanned<'a> for NodeRef<'a, GenericType<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> Identifiable<'a> for NodeRef<'a, GenericType<'a>> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a> {
        *ctx[*self].name
    }
}

impl<'a> Typed<'a> for NodeRef<'a, GenericType<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let name = *ctx[*self].name;

        Ok(ctx.get_type_sig(
            TypeSignatureValue::Generic { name },
            TypeSignatureContext {
                parent: TypeSignatureParent::GenericType(*self),
                type_span: name.get_span(ctx),
            }
            .alloc(),
        ))
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::generics::GenericsDecl<'a> {
    type IrType = GenericsDecl<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        GenericsDecl {
            generics: self
                .generics
                .into_iter()
                .map(|val| val.ir_lower(ctx))
                .collect(),
            span: self.span,
        }
        .allocate(ctx)
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::generics::GenericType<'a> {
    type IrType = GenericType<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let gen_val = GenericType {
            name: LateInit::empty(),
            span: self.span,
        }
        .allocate(ctx);

        ctx[gen_val].name = ctx
            .make_ident(self.name, IdentParent::GenericValueName(gen_val))
            .into();

        gen_val
    }
}
