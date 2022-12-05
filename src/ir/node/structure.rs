use crate::{
    error_message::error_formatter::Spanned,
    ir::{ast_lowering::IrLowerable, context::IrCtx, late_init::LateInit},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    expression::Expr,
    identifier::{Ident, IdentKey, IdentParent, Identifiable},
    type_signature::{
        Mutability, TypeEvalError, TypeSignature, TypeSignatureContext, TypeSignatureParent,
        TypeSignatureValue, Typed,
    },
    IrAlloc, NodeRef,
};

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: LateInit<Ident<'a>>,
    pub attrs: Vec<NodeRef<'a, StructAttr<'a>>>,
}

#[derive(Debug)]
pub struct StructAttr<'a> {
    pub name: LateInit<Ident<'a>>,
    pub mutability: Mutability,
    pub type_sig: LateInit<TypeSignature<'a>>,
    pub default_value: Option<NodeRef<'a, Expr<'a>>>,
}

#[derive(Debug)]
pub struct StructInit<'a> {
    pub type_sig: LateInit<TypeSignature<'a>>,
    pub scope_name: LateInit<Ident<'a>>,
    pub values: Vec<NodeRef<'a, StructInitValue<'a>>>,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub struct StructInitValue<'a> {
    pub name: LateInit<Ident<'a>>,
    pub parent: NodeRef<'a, StructInit<'a>>,
    pub value: NodeRef<'a, Expr<'a>>,
}

#[derive(Debug)]
pub struct StructAccess<'a> {
    pub struct_expr: NodeRef<'a, Expr<'a>>,
    pub attr_name: Ident<'a>,
}

impl<'a> Spanned<'a> for NodeRef<'a, StructAccess<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        ctx[*self].attr_name.get_span(ctx)
    }
}

impl<'a> NodeRef<'a, Struct<'a>> {
    pub fn lookup_attr(
        &self,
        ident: Ident<'a>,
        ctx: &IrCtx<'a>,
    ) -> Option<NodeRef<'a, StructAttr<'a>>> {
        ctx[*self]
            .attrs
            .iter()
            .find(|attr| IdentKey::idents_eq(ctx, *ctx[**attr].name, ident))
            .map(|attr| *attr)
    }
}

impl<'a> Identifiable<'a> for Struct<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.name
    }
}

impl<'a> Identifiable<'a> for StructAttr<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.name
    }
}

impl<'a> Identifiable<'a> for StructInit<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.scope_name
    }
}

impl<'a> Spanned<'a> for NodeRef<'a, Struct<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        ctx[*self].name.get_span(ctx)
    }
}

impl<'a> Spanned<'a> for NodeRef<'a, StructAttr<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        ctx[*self].name.get_span(ctx)
    }
}

impl<'a> Spanned<'a> for NodeRef<'a, StructInit<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, Struct<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let name = *ctx[*self].name;
        Ok(ctx.get_type_sig(
            TypeSignatureValue::Struct { name },
            TypeSignatureContext {
                parent: TypeSignatureParent::Struct(*self),
                type_span: None,
            }
            .alloc(),
        ))
    }
}

impl<'a> Typed<'a> for NodeRef<'a, StructAttr<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match ctx[*self].default_value {
            Some(value) => value.eval_type(symbols, ctx),
            None => {
                let type_sig = (*ctx[*self].type_sig).clone();
                debug_assert!(!matches!(ctx[&type_sig], TypeSignatureValue::Unresolved(_)));
                Ok(type_sig)
            }
        }
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

impl<'a> IrLowerable<'a> for crate::ast::node::structure::Struct<'a> {
    type IrType = Struct<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let ir_attrs: Vec<NodeRef<'a, StructAttr<'a>>> = self
            .attrs
            .into_iter()
            .map(|attr| attr.ir_lower(ctx))
            .collect();

        let st = Struct {
            name: LateInit::empty(),
            attrs: ir_attrs,
        }
        .allocate(ctx);

        ctx[st].name = ctx
            .make_ident(self.name, IdentParent::StructDeclName(st))
            .into();

        st
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::structure::StructAttr<'a> {
    type IrType = StructAttr<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let st_attr = StructAttr {
            name: LateInit::empty(),
            mutability: self.mutability,
            type_sig: LateInit::empty(),
            default_value: self.default_value.map(|val| val.ir_lower(ctx)),
        }
        .allocate(ctx);

        ctx[st_attr].name = ctx
            .make_ident(self.name, IdentParent::StructDeclAttrName(st_attr))
            .into();

        ctx[st_attr].type_sig = self
            .type_sig
            .map(|t| t.into_ir_type(ctx, TypeSignatureParent::StructAttr(st_attr)))
            .unwrap_or_else(|| ctx.make_type_var(TypeSignatureParent::StructAttr(st_attr)))
            .into();

        st_attr
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::structure::StructInit<'a> {
    type IrType = StructInit<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let struct_init = StructInit {
            type_sig: LateInit::empty(),
            scope_name: LateInit::empty(),
            values: Vec::new(),
            span: self.span,
        }
        .allocate(ctx);

        ctx[struct_init].type_sig = self
            .struct_name
            .map(|struct_name| {
                let st_ident = ctx.make_unresolved_ident(
                    struct_name.clone(),
                    IdentParent::StructInitStructName(struct_init).into(),
                );
                ctx.get_type_sig(
                    TypeSignatureValue::Unresolved(st_ident),
                    TypeSignatureContext {
                        parent: TypeSignatureParent::StructInit(struct_init),
                        type_span: Some(struct_name.span.clone()),
                    }
                    .alloc(),
                )
            })
            .unwrap_or_else(|| ctx.make_type_var(TypeSignatureParent::StructInit(struct_init)))
            .into();

        ctx[struct_init].scope_name = ctx
            .make_anon_ident(IdentParent::StructInitScopeName(struct_init))
            .into();

        let st_init_vals = self
            .values
            .into_iter()
            .map(|val| {
                let st_val = StructInitValue {
                    name: LateInit::empty(),
                    parent: struct_init,
                    value: val.value.ir_lower(ctx),
                }
                .allocate(ctx);

                ctx[st_val].name = ctx
                    .make_unresolved_ident(
                        val.name,
                        IdentParent::StructInitValueName(st_val).into(),
                    )
                    .into();

                st_val
            })
            .collect();

        ctx[struct_init].values = st_init_vals;

        struct_init
    }
}

impl<'a> NodeRef<'a, StructInit<'a>> {
    pub fn lookup_struct(
        &self,
        ctx: &IrCtx<'a>,
        symbols: &SymbolTableZipper<'a>,
    ) -> Option<NodeRef<'a, Struct<'a>>> {
        let Some(struct_name) = self.struct_name(ctx) else {
            return None;
        };

        symbols
            .lookup(ctx, struct_name)
            .map(|val| val.unwrap_struct(ctx))
    }
}

impl<'a> Typed<'a> for NodeRef<'a, StructInit<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok((*ctx[*self].type_sig).clone())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, StructAccess<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        self.lookup_attr(ctx, symbols)?.eval_type(symbols, ctx)
    }
}

impl<'a> NodeRef<'a, StructAccess<'a>> {
    pub fn lookup_attr(
        &self,
        ctx: &mut IrCtx<'a>,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<NodeRef<'a, StructAttr<'a>>, TypeEvalError<'a>> {
        let st_type = ctx[*self].struct_expr.clone().eval_type(symbols, ctx)?;
        let struct_name = match &ctx[&st_type] {
            TypeSignatureValue::Struct { name } => *name,
            _ => return Err(TypeEvalError::AccessNonStruct(st_type)),
        };

        let st = symbols
            .lookup(ctx, struct_name)
            .ok_or(TypeEvalError::UnknownIdent(struct_name))?
            .unwrap_struct(ctx);

        let attr_name = ctx[*self].attr_name;
        st.lookup_attr(attr_name, ctx)
            .ok_or(TypeEvalError::UnknownIdent(attr_name))
    }

    pub fn lookup_attr_chain<'c>(
        &self,
        ctx: &mut IrCtx<'a>,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<Vec<NodeRef<'a, StructAttr<'a>>>, TypeEvalError<'a>> {
        fn recursive_lookup<'a>(
            result: &mut Vec<NodeRef<'a, StructAttr<'a>>>,
            st_access: NodeRef<'a, StructAccess<'a>>,
            ctx: &mut IrCtx<'a>,
            symbols: &mut SymbolTableZipper<'a>,
        ) -> Result<(), TypeEvalError<'a>> {
            let inner_expr = ctx[st_access].struct_expr;
            if let Expr::StructAccess(inner_access) = &ctx[inner_expr] {
                recursive_lookup(result, *inner_access, ctx, symbols)?;
            }

            let attr = st_access.lookup_attr(ctx, symbols)?;
            result.push(attr);

            Ok(())
        }

        let mut result = Vec::new();
        recursive_lookup(&mut result, *self, ctx, symbols)?;

        Ok(result)
    }
}

impl<'a> NodeRef<'a, StructInit<'a>> {
    pub fn struct_name(&self, ctx: &IrCtx<'a>) -> Option<Ident<'a>> {
        match &ctx[&*ctx[*self].type_sig] {
            TypeSignatureValue::Struct { name } => Some(*name),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::test_utils::utils::{lowered_ir, type_check};

    #[test]
    fn test_nested_struct() {
        let mut ir = lowered_ir(
            "
        struct Deep {
            var inner = false
        }

        struct Foo {
            var bar: Deep
        }

        var foo = Foo { bar: Deep {} }
        foo.bar.inner = true
        ",
        )
        .unwrap();

        assert_matches!(type_check(&mut ir).1, Ok(_))
    }
}
