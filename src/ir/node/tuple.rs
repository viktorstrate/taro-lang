use crate::{
    error_message::error_formatter::Spanned,
    ir::{ast_lowering::IrLowerable, context::IrCtx, late_init::LateInit},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    expression::Expr,
    type_signature::{
        TypeEvalError, TypeSignature, TypeSignatureContext, TypeSignatureParent,
        TypeSignatureValue, Typed,
    },
    IrAlloc, NodeRef,
};

#[derive(Debug)]
pub struct Tuple<'a> {
    pub values: Vec<NodeRef<'a, Expr<'a>>>,
    pub type_sig: LateInit<TypeSignature<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub struct TupleAccess<'a> {
    pub tuple_expr: NodeRef<'a, Expr<'a>>,
    pub attr: usize,
    pub span: Span<'a>,
}

impl<'a> Spanned<'a> for NodeRef<'a, TupleAccess<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> Spanned<'a> for NodeRef<'a, Tuple<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, Tuple<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let types = ctx[*self]
            .values
            .clone()
            .into_iter()
            .map(|val| val.eval_type(symbols, ctx))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ctx.get_type_sig(
            TypeSignatureValue::Tuple(types.into()),
            TypeSignatureContext {
                parent: TypeSignatureParent::Tuple(*self),
                type_span: None,
            }
            .alloc(),
        ))
    }

    fn specified_type(&self, ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        Some((*ctx[*self].type_sig).clone())
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        match &ctx[&new_type] {
            TypeSignatureValue::Tuple(vals) => {
                assert_eq!(vals.len(), ctx[*self].values.len(), "tuple length match")
            }
            _ => assert!(false),
        }

        ctx[*self].type_sig = new_type.into();
        Ok(())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, TupleAccess<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let tuple_type = ctx[*self].tuple_expr.clone().eval_type(symbols, ctx)?;
        let attr = ctx[*self].attr;
        match &ctx[&tuple_type] {
            TypeSignatureValue::Tuple(tuple) => {
                tuple
                    .get(attr)
                    .cloned()
                    .ok_or(TypeEvalError::TupleAccessOutOfBounds {
                        tuple_len: tuple.len(),
                        access_item: attr,
                    })
            }
            _val => Err(TypeEvalError::AccessNonTuple(tuple_type)),
        }
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::tuple::Tuple<'a> {
    type IrType = Tuple<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let tup = Tuple {
            values: self
                .values
                .into_iter()
                .map(|val| val.ir_lower(ctx))
                .collect(),
            type_sig: LateInit::empty(),
            span: self.span,
        }
        .allocate(ctx);

        ctx[tup].type_sig = self
            .type_sig
            .map(|t| t.into_ir_type(ctx, TypeSignatureParent::Tuple(tup)))
            .unwrap_or_else(|| ctx.make_type_var(TypeSignatureParent::Tuple(tup)))
            .into();

        tup
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::tuple::TupleAccess<'a> {
    type IrType = TupleAccess<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        TupleAccess {
            tuple_expr: self.tuple_expr.ir_lower(ctx),
            attr: self.attr,
            span: self.span,
        }
        .allocate(ctx)
    }
}
