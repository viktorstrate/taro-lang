use crate::{
    ir::{
        context::IrCtx,
        ir_walker::{IrWalker, ScopeValue},
        node::{
            expression::Expr,
            function::Function,
            statement::{Stmt, StmtBlock},
            type_signature::{
                BuiltinType, TypeEvalError, TypeSignature, TypeSignatureValue, Typed,
            },
            NodeRef,
        },
    },
    symbols::symbol_resolver::SymbolResolutionError,
};

use super::{coercion::coerce, FunctionError, TypeChecker, TypeCheckerError};

#[derive(Debug)]
pub struct TypeConstraint<'a>(pub TypeSignature<'a>, pub TypeSignature<'a>);

#[derive(Debug)]
pub struct TypeInferrer<'a, 'b>(pub &'b mut TypeChecker<'a>);

impl<'a, 'b> TypeInferrer<'a, 'b> {
    pub fn new(ctx: &IrCtx<'a>, type_checker: &'b mut TypeChecker<'a>) -> Self {
        type_checker.symbols.reset(ctx);
        Self(type_checker)
    }
}

impl<'a> IrWalker<'a> for TypeInferrer<'a, '_> {
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

    fn visit_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        self.resolve_constraints(ctx)?;
        Ok(())
    }

    fn visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        stmt: NodeRef<'a, Stmt<'a>>,
    ) -> Result<(), Self::Error> {
        match ctx[stmt].clone() {
            Stmt::VariableDecl(var_decl) => {
                let val_type = ctx[var_decl]
                    .value
                    .clone()
                    .eval_type(&mut self.0.symbols, ctx)
                    .map_err(TypeCheckerError::TypeEval)?;

                self.0
                    .add_constraint((*ctx[var_decl].type_sig).clone(), val_type)
            }
            Stmt::StructDecl(st) => {
                for attr in ctx[st].attrs.clone() {
                    if let Some(attr_val) = ctx[attr].default_value {
                        let default_val_type = attr_val
                            .eval_type(&mut self.0.symbols, ctx)
                            .map_err(TypeCheckerError::TypeEval)?;

                        self.0
                            .add_constraint((*ctx[attr].type_sig).clone(), default_val_type)
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), Self::Error> {
        match ctx[expr].clone() {
            Expr::FunctionCall(call) => {
                let func_type_sig = ctx[call]
                    .func
                    .clone()
                    .eval_type(&mut self.0.symbols, ctx)
                    .map_err(TypeCheckerError::TypeEval)?;

                let args = match &ctx[&func_type_sig] {
                    TypeSignatureValue::Function {
                        args,
                        return_type: _,
                    } => args.clone(),
                    _ => {
                        return Err(TypeCheckerError::TypeEval(TypeEvalError::CallNonFunction(
                            call,
                            func_type_sig,
                        )));
                    }
                };

                let func_params = ctx[call].args.clone();

                if args.len() != func_params.len() {
                    return Err(TypeCheckerError::FunctionError(
                        FunctionError::FuncCallWrongArgAmount {
                            call,
                            func_type: func_type_sig,
                        },
                    ));
                }

                for (arg, param) in (*args).clone().into_iter().zip(func_params.into_iter()) {
                    let param_type = param
                        .eval_type(&mut self.0.symbols, ctx)
                        .map_err(TypeCheckerError::TypeEval)?;

                    self.0.add_constraint(arg, param_type)
                }
            }
            Expr::Identifier(_, _) => {}
            Expr::StructInit(st_init) => {
                let Some(st_name) = st_init.struct_name(ctx) else {
                    return Ok(())
                };

                let st = self
                    .0
                    .symbols
                    .lookup(ctx, st_name)
                    .ok_or(TypeCheckerError::TypeEval(TypeEvalError::UnknownIdent(
                        st_name,
                    )))?
                    .unwrap_struct(ctx);

                for val in ctx[st_init].values.clone() {
                    let st_attr =
                        st.lookup_attr(*ctx[val].name, ctx)
                            .ok_or(TypeCheckerError::TypeEval(TypeEvalError::UnknownIdent(
                                *ctx[val].name,
                            )))?;

                    let val_type = ctx[val]
                        .value
                        .clone()
                        .eval_type(&mut self.0.symbols, ctx)
                        .map_err(TypeCheckerError::TypeEval)?;

                    self.0
                        .add_constraint((*ctx[st_attr].type_sig).clone(), val_type);
                }
            }
            Expr::StructAccess(_) => {}
            Expr::TupleAccess(_) => {}
            Expr::EscapeBlock(_) => {}
            Expr::Assignment(asg) => {
                let lhs = ctx[asg]
                    .lhs
                    .clone()
                    .eval_type(&mut self.0.symbols, ctx)
                    .map_err(TypeCheckerError::TypeEval)?;
                let rhs = ctx[asg]
                    .rhs
                    .clone()
                    .eval_type(&mut self.0.symbols, ctx)
                    .map_err(TypeCheckerError::TypeEval)?;

                self.0.add_constraint(lhs, rhs);
            }
            Expr::Tuple(_) => {}
            Expr::EnumInit(enm_init) => {
                let enm = enm_init.lookup_enum(ctx, &mut self.0.symbols).ok_or(
                    TypeCheckerError::SymbolResolutionError(SymbolResolutionError::TypeEval(
                        TypeEvalError::UnknownIdent(ctx[enm_init].enum_name),
                    )),
                )?;
                let enm_val = enm
                    .lookup_value(ctx, ctx[enm_init].enum_value)
                    .ok_or(TypeCheckerError::SymbolResolutionError(
                        SymbolResolutionError::TypeEval(TypeEvalError::UnknownIdent(
                            ctx[enm_init].enum_name,
                        )),
                    ))?
                    .1;

                for (arg, item_type) in ctx[enm_init]
                    .items
                    .clone()
                    .into_iter()
                    .zip((*ctx[enm_val].items).clone().into_iter())
                {
                    let arg_type = arg
                        .eval_type(&mut self.0.symbols, ctx)
                        .map_err(TypeCheckerError::TypeEval)?;
                    self.0.add_constraint(arg_type, item_type);
                }
            }
            Expr::UnresolvedMemberAccess(_) => {}
            _ => {}
        }

        Ok(())
    }

    fn visit_func_decl(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        func: NodeRef<'a, Function<'a>>,
    ) -> Result<(), Self::Error> {
        self.infer_function_body(ctx, func)?;
        Ok(())
    }
}

impl<'a> TypeInferrer<'a, '_> {
    fn resolve_constraints(&mut self, ctx: &mut IrCtx<'a>) -> Result<(), TypeCheckerError<'a>> {
        let mut unresolvable_count = 0;
        while let Some(TypeConstraint(type_a, type_b)) = self.0.constraints.pop_front() {
            let type_a = (*self.0.substitutions.get(&type_a).unwrap_or(&type_a)).clone();
            let type_b = (*self.0.substitutions.get(&type_b).unwrap_or(&type_b)).clone();

            // println!(
            //     "TYPE CONSTRAINT :: {} == {}",
            //     type_a.format(ctx),
            //     type_b.format(ctx)
            // );

            match (ctx[&type_a].clone(), ctx[&type_b].clone()) {
                (TypeSignatureValue::TypeVariable(_), TypeSignatureValue::TypeVariable(_)) => {
                    if unresolvable_count < self.0.constraints.len() {
                        unresolvable_count += 1;
                        self.0.add_constraint(type_a, type_b);
                    } else {
                        // No more constraints can be resolved
                        self.0.add_constraint(type_a, type_b);
                        return Ok(());
                    }
                }
                (TypeSignatureValue::TypeVariable(_), _) => {
                    unresolvable_count = 0;
                    self.0.substitutions.insert(type_a, type_b);
                }
                (_, TypeSignatureValue::TypeVariable(_)) => {
                    unresolvable_count = 0;
                    self.0.substitutions.insert(type_b, type_a);
                }
                (TypeSignatureValue::Tuple(tup_a), TypeSignatureValue::Tuple(tup_b)) => {
                    unresolvable_count = 0;
                    if tup_a.len() != tup_b.len() {
                        return Err(TypeCheckerError::ConflictingTypes(type_a, type_b));
                    }
                    for (val_a, val_b) in (*tup_a).iter().zip((*tup_b).iter()) {
                        self.0.add_constraint(val_a.clone(), val_b.clone());
                    }
                }
                (
                    TypeSignatureValue::Function {
                        args: args_a,
                        return_type: return_type_a,
                    },
                    TypeSignatureValue::Function {
                        args: args_b,
                        return_type: return_type_b,
                    },
                ) => {
                    if args_a.len() != args_b.len() {
                        return Err(TypeCheckerError::FunctionError(
                            FunctionError::ArgCountMismatch(type_a, type_b),
                        ));
                    }

                    self.0
                        .add_constraint((*return_type_a).clone(), (*return_type_b).clone());
                    for (arg_a, arg_b) in (*args_a)
                        .clone()
                        .into_iter()
                        .zip((*args_b).clone().into_iter())
                    {
                        self.0.add_constraint(arg_a, arg_b);
                    }
                }
                _ => {
                    unresolvable_count = 0;
                    if coerce(&type_a, &type_b, ctx).is_none() {
                        // println!(
                        //     "CONFLICTING TYPES: {} /= {}",
                        //     type_a.format(ctx),
                        //     type_b.format(ctx)
                        // );
                        return Err(TypeCheckerError::ConflictingTypes(type_a, type_b));
                    }
                }
            }
        }

        Ok(())
    }

    fn infer_function_body(
        &mut self,
        ctx: &mut IrCtx<'a>,
        func: NodeRef<'a, Function<'a>>,
    ) -> Result<(), TypeCheckerError<'a>> {
        fn collect_return_types<'a>(
            inferrer: &mut TypeInferrer<'a, '_>,
            ctx: &mut IrCtx<'a>,
            stmt_block: NodeRef<'a, StmtBlock<'a>>,
            acc: &mut Vec<TypeSignature<'a>>,
        ) -> Result<(), TypeCheckerError<'a>> {
            for stmt in ctx[stmt_block].0.clone() {
                match ctx[stmt] {
                    Stmt::Return(expr) => {
                        let expr_type = expr
                            .eval_type(&mut inferrer.0.symbols, ctx)
                            .map_err(TypeCheckerError::TypeEval)?;
                        acc.push(expr_type)
                    }
                    Stmt::IfBranch(ifb) => {
                        collect_return_types(inferrer, ctx, ctx[ifb].body, acc)?;
                        if let Some(else_block) = ctx[ifb].else_body {
                            collect_return_types(inferrer, ctx, else_block, acc)?;
                        }
                    }
                    _ => {}
                };
            }

            Ok(())
        }

        let mut return_types = Vec::new();
        return_types.push((*ctx[func].return_type).clone());
        collect_return_types(self, ctx, ctx[func].body, &mut return_types)?;

        if return_types.len() == 1 {
            self.0.add_constraint(
                return_types[0].clone(),
                ctx.get_builtin_type_sig(BuiltinType::Void),
            )
        }

        for i in 1..return_types.len() {
            self.0
                .add_constraint(return_types[i - 1].clone(), return_types[i].clone());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::{
        node::type_signature::BuiltinType,
        test_utils::utils::{lowered_ir, type_check},
    };

    use super::*;

    fn assert_type_mismatch<'a>(
        (_, infer_result): (TypeChecker<'a>, Result<(), TypeCheckerError<'a>>),
        type_a: TypeSignature<'a>,
        type_b: TypeSignature<'a>,
    ) {
        match infer_result {
            Err(TypeCheckerError::ConflictingTypes(a, b)) => {
                assert!(type_a == a || type_a == b, "expected A did not match");
                assert!(type_b == a || type_b == b, "expected B did not match");
                assert!((a == type_a && b == type_b) || (a == type_b && b == type_a));
            }
            _ => assert!(
                false,
                "expected conflicting type error, got {infer_result:?}"
            ),
        }
    }

    #[test]
    fn test_var_decl_mismatched_types() {
        let mut ir = lowered_ir("let x: String = 2").unwrap();

        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::String),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
        );
    }

    #[test]
    fn test_assign_variable_types_mismatch() {
        let mut ir = lowered_ir("var foo = 1; foo = false").unwrap();
        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }

    #[test]
    fn test_var_decl_var() {
        let mut ir = lowered_ir("let a = true; let b: Boolean = a").unwrap();
        assert_matches!(type_check(&mut ir), (_, Ok(_)));
    }

    #[test]
    fn test_struct_access_mismatched_types() {
        let mut ir = lowered_ir(
            "\
        struct Test { let attr: Number }
        let test = Test { attr: 123 }
        let wrong: Boolean = test.attr",
        )
        .unwrap();

        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
        );
    }

    #[test]
    fn test_struct_decl_attr_mismatched_types() {
        let mut ir = lowered_ir("struct Test { let attr: String = true }").unwrap();

        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::String),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }

    #[test]
    fn test_unknown_member_access() {
        let mut ir = lowered_ir(".x").unwrap();
        let (_, result) = type_check(&mut ir);

        match result {
            Ok(_) => assert!(false),
            Err(err) => match err {
                TypeCheckerError::UndeterminableTypes(types) => {
                    assert_eq!(types.len(), 1)
                }
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn test_func_decl_inside_struct() {
        let mut ir = lowered_ir(
            "struct Foo { let attr: () -> Number }
            let a = Foo { attr: () { return false } }",
        )
        .unwrap();

        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }

    #[test]
    fn test_call_non_function() {
        let mut ir = lowered_ir("let val = true; val()").unwrap();

        match type_check(&mut ir).1 {
            Err(TypeCheckerError::TypeEval(TypeEvalError::CallNonFunction(_, expr_type))) => {
                assert_eq!(expr_type, ir.ctx.get_builtin_type_sig(BuiltinType::Boolean))
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_assign_func_call_mismatched_types() {
        let mut ir = lowered_ir("func f() { return 123 }; let x: Boolean = f()").unwrap();
        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }

    #[test]
    fn test_func_call_wrong_arg_type() {
        let mut ir = lowered_ir("func f(a: Number) {}; f(true)").unwrap();
        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }

    #[test]
    fn test_func_call_wrong_arg_amount() {
        let mut ir = lowered_ir("func f(a: Number) {}; f(2, 3)").unwrap();
        assert_matches!(
            type_check(&mut ir).1,
            Err(TypeCheckerError::FunctionError(
                FunctionError::FuncCallWrongArgAmount {
                    call: _,
                    func_type: _
                }
            ))
        );
    }

    #[test]
    fn test_func_return_typecheck() {
        let mut ir = lowered_ir("func test() -> Number { return false }").unwrap();
        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }

    #[test]
    fn test_func_return_implicit_void() {
        let mut ir = lowered_ir("func test() -> Number { }").unwrap();
        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
            ir.ctx.get_builtin_type_sig(BuiltinType::Void),
        );
    }

    #[test]
    fn test_inconsistent_if_returns() {
        let mut ir =
            lowered_ir("func test() -> Number { if true { return 1 } else { return false } }")
                .unwrap();
        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }

    #[test]
    fn test_escape_block_typed() {
        let mut ir = lowered_ir("let a: Number = @{ 1 + 2 }").unwrap();
        assert_matches!(type_check(&mut ir).1, Ok(_));
    }

    #[test]
    fn test_escape_block_untyped() {
        let mut ir = lowered_ir("let a = @{ 1 + 2 }").unwrap();
        assert_matches!(
            type_check(&mut ir).1,
            Err(TypeCheckerError::UnresolvableTypeConstraints(_))
        );
    }

    #[test]
    fn test_escape_block_untyped_func() {
        let mut ir = lowered_ir("func foo() { return @{ 123 } }").unwrap();
        assert_matches!(
            type_check(&mut ir).1,
            Err(TypeCheckerError::UnresolvableTypeConstraints(_))
        );
    }

    #[test]
    fn test_external_object() {
        let mut ir =
            lowered_ir("external foo: (Number) -> String; let x: Boolean = foo(2)").unwrap();
        assert_type_mismatch(
            type_check(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::String),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }
}
