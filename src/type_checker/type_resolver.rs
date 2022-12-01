use crate::{
    error_message::error_formatter::Spanned,
    ir::{
        context::IrCtx,
        ir_walker::{IrWalker, ScopeValue},
        node::{
            enumeration::EnumInit,
            expression::Expr,
            identifier::{Ident, IdentParent, Identifiable},
            member_access::UnresolvedMemberAccess,
            type_signature::{TypeEvalError, TypeSignature, TypeSignatureValue},
            IrAlloc, NodeRef,
        },
    },
    symbols::symbol_resolver::SymbolResolutionError,
};

use super::{
    type_inference::TypeInferrer, ExpectedType, TypeChecker, TypeCheckerError, UndeterminableType,
};

#[derive(Debug)]
pub struct TypeResolver<'a, 'b>(pub &'b mut TypeChecker<'a>);

impl<'a> IrWalker<'a> for TypeResolver<'a, '_> {
    type Error = TypeCheckerError<'a>;
    type Scope = ();

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        value.visit_scope_begin(ctx, &mut self.0.symbols);
        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        _child: Self::Scope,
        _value: ScopeValue<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        self.0
            .symbols
            .exit_scope(ctx)
            .expect("scope should not be global scope");

        Ok(())
    }

    fn visit_ordered_symbol(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        self.0.symbols.visit_next_symbol(ctx);
        Ok(())
    }

    fn visit_type_sig(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        type_sig: TypeSignature<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        let new_type = self.0.substitutions.get(&type_sig).cloned();

        if let Some(t) = &new_type {
            // struct_init uses type_sig to resolve the struct definition which can be used to infer the attributes
            if let TypeSignatureValue::Struct { name: _ } = ctx[t] {
                self.0.needs_rerun = true;
            }
        }

        Ok(new_type.unwrap_or(type_sig))
    }

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), Self::Error> {
        match ctx[expr] {
            Expr::UnresolvedMemberAccess(mem_acc) => self.resolve_member_access(ctx, mem_acc, expr),
            _ => Ok(()),
        }
    }
}

impl<'a, 'b> TypeResolver<'a, 'b> {
    pub fn new(
        ctx: &IrCtx<'a>,
        type_inferrer: &'b mut TypeInferrer<'a, '_>,
    ) -> TypeResolver<'a, 'b> {
        type_inferrer.0.symbols.reset(ctx);

        TypeResolver(type_inferrer.0)
    }

    fn resolve_member_access(
        &mut self,
        ctx: &mut IrCtx<'a>,
        mem_acc: NodeRef<'a, UnresolvedMemberAccess<'a>>,
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), TypeCheckerError<'a>> {
        // let mem_acc = match &ctx[expr] {
        //     Expr::UnresolvedMemberAccess(mem_acc) => *mem_acc,
        //     _ => return Ok(()),
        // };

        // Make sure type sig is resolved before proceeding
        ctx[mem_acc].type_sig = self
            .visit_type_sig(ctx, &mut (), (*ctx[mem_acc].type_sig).clone())?
            .into();

        let enm_init = match &ctx[&*ctx[mem_acc].type_sig] {
            TypeSignatureValue::Enum { name } => {
                let items = match ctx[mem_acc].items.clone() {
                    Some((items, span)) => (Some(items), Some(span)),
                    None => (None, None),
                };

                EnumInit {
                    enum_name: *name,
                    enum_value: *ctx[mem_acc].member_name,
                    items: items.0.unwrap_or_default(),
                    items_span: items.1,
                    span: ctx[mem_acc].span.clone(),
                }
                .allocate(ctx)
            }
            TypeSignatureValue::TypeVariable(_) => {
                let span = (*ctx[mem_acc].type_sig).get_span(ctx).unwrap();

                if !self
                    .0
                    .previous_undeterminable_types
                    .iter()
                    .find(|x| x.span == span)
                    .is_some()
                {
                    self.0.needs_rerun = true;
                }

                self.0
                    .immediate_undeterminable_types
                    .push(UndeterminableType {
                        span,
                        expected: ExpectedType::Enum,
                    });

                return Ok(());
            }
            _ => {
                return Err(TypeCheckerError::AnonymousEnumInitNonEnum(
                    mem_acc,
                    (*ctx[mem_acc].type_sig).clone(),
                ))
            }
        };

        let enm_expr = Expr::EnumInit(enm_init);

        ctx[expr] = enm_expr;

        // Symbol resolve enum name and value
        let enm_name = ctx[enm_init].enum_name;
        let enm = self
            .0
            .symbols
            .lookup(ctx, enm_name)
            .ok_or(TypeCheckerError::SymbolResolutionError(
                SymbolResolutionError::TypeEval(TypeEvalError::UnknownIdent(enm_name)),
            ))?
            .unwrap_enum(ctx);
        let (_, enm_val) = enm.lookup_value(ctx, ctx[enm_init].enum_value).ok_or(
            TypeCheckerError::SymbolResolutionError(SymbolResolutionError::UnknownEnumValue {
                enm,
                enum_value: ctx[enm_init].enum_value,
            }),
        )?;
        ctx[enm_init].enum_value = *ctx[enm_val].name;

        let sym_id =
            self.0
                .symbols
                .lookup(ctx, enm_name)
                .ok_or(TypeCheckerError::SymbolResolutionError(
                    SymbolResolutionError::TypeEval(TypeEvalError::UnknownIdent(enm_name)),
                ))?;

        ctx[enm_init].enum_name = Ident {
            id: (*&ctx[sym_id]).name(ctx).id,
            parent: IdentParent::EnumInitEnumName(enm_init).into(),
        };

        // New types can now potentially be inferred
        self.0.needs_rerun = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::test_utils::utils::{lowered_ir, type_check};

    #[test]
    fn test_unresolved_struct_name() {
        let mut ir = lowered_ir(
            "
        struct Foo {
            let hello: String
        }

        let a: Foo = { hello: \"world\" }
        ",
        )
        .unwrap();
        assert_matches!(type_check(&mut ir).1, Ok(_));
    }

    #[test]
    fn test_unresolved_nested_struct_name() {
        let mut ir = lowered_ir(
            "
        struct Foo {
            let bar: Bar
        }

        struct Bar {
            let val: Number
        }

        let a: Foo = { bar: { val: 42 } }
        ",
        )
        .unwrap();
        assert_matches!(type_check(&mut ir).1, Ok(_));
    }
}
