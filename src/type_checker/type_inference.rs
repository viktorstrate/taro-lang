use std::collections::HashMap;

use crate::{
    ir::{
        context::IrCtx,
        ir_walker::{IrWalker, ScopeValue},
        node::{
            expression::Expr,
            statement::Stmt,
            type_signature::{TypeEvalError, TypeSignature, TypeSignatureValue, Typed},
            NodeRef,
        },
    },
    symbols::{
        symbol_resolver::SymbolResolver, symbol_table::symbol_table_zipper::SymbolTableZipper,
    },
};

use super::coercion::coerce;

#[derive(Debug)]
pub struct TypeConstraint<'a>(TypeSignature<'a>, TypeSignature<'a>);

#[derive(Debug)]
pub struct TypeInferrer<'a> {
    pub symbols: SymbolTableZipper<'a>,
    pub substitutions: HashMap<TypeSignature<'a>, TypeSignature<'a>>,
    pub constraints: Vec<TypeConstraint<'a>>,
}

#[derive(Debug)]
pub enum TypeInferenceError<'a> {
    ConflictingTypes(TypeSignature<'a>, TypeSignature<'a>),
    UndeterminableTypes,
    TypeEval(TypeEvalError<'a>),
}

impl<'a> TypeInferrer<'a> {
    pub fn new(ctx: &IrCtx<'a>, sym_resolver: SymbolResolver<'a>) -> Self {
        let mut symbols = sym_resolver.symbols;
        symbols.reset(ctx);
        TypeInferrer {
            symbols,
            substitutions: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    #[inline(always)]
    fn add_constraint(&mut self, a: TypeSignature<'a>, b: TypeSignature<'a>) {
        self.constraints.push(TypeConstraint(a, b))
    }
}

impl<'a> IrWalker<'a> for TypeInferrer<'a> {
    type Error = TypeInferenceError<'a>;
    type Scope = ();

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), TypeInferenceError<'a>> {
        value.visit_scope_begin(ctx, &mut self.symbols);
        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        _child: Self::Scope,
        _value: ScopeValue<'a>,
    ) -> Result<(), TypeInferenceError<'a>> {
        self.symbols
            .exit_scope(ctx)
            .expect("scope should not be global scope");

        Ok(())
    }

    fn visit_ordered_symbol(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        self.symbols.visit_next_symbol(ctx);
        Ok(())
    }

    fn visit_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        while let Some(TypeConstraint(type_a, type_b)) = self.constraints.pop() {
            match (ctx[type_a].clone(), ctx[type_b].clone()) {
                (TypeSignatureValue::TypeVariable(_), TypeSignatureValue::TypeVariable(_)) => {
                    return Err(TypeInferenceError::UndeterminableTypes);
                }
                (TypeSignatureValue::TypeVariable(_), _) => {
                    self.substitutions.insert(type_b, type_a);
                }
                (_, TypeSignatureValue::TypeVariable(_)) => {
                    self.substitutions.insert(type_a, type_b);
                }
                (TypeSignatureValue::Tuple(tup_a), TypeSignatureValue::Tuple(tup_b)) => {
                    if tup_a.len() != tup_b.len() {
                        return Err(TypeInferenceError::ConflictingTypes(type_a, type_b));
                    }
                    for (val_a, val_b) in tup_a.into_iter().zip(tup_b.into_iter()) {
                        self.constraints.push(TypeConstraint(val_a, val_b));
                    }
                }
                _ => {
                    if coerce(type_a, type_b, ctx).is_none() {
                        return Err(TypeInferenceError::ConflictingTypes(type_a, type_b));
                    }
                }
            }
        }

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
                    .eval_type(&mut self.symbols, ctx)
                    .map_err(TypeInferenceError::TypeEval)?;

                self.add_constraint(ctx[var_decl].type_sig, val_type)
            }
            Stmt::StructDecl(st) => {
                for attr in ctx[st].attrs.clone() {
                    if let Some(attr_val) = ctx[attr].default_value {
                        let default_val_type = attr_val
                            .eval_type(&mut self.symbols, ctx)
                            .map_err(TypeInferenceError::TypeEval)?;

                        self.add_constraint(ctx[attr].type_sig, default_val_type)
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
                    .eval_type(&mut self.symbols, ctx)
                    .map_err(TypeInferenceError::TypeEval)?;

                let args = match &ctx[func_type_sig] {
                    TypeSignatureValue::Function {
                        args,
                        return_type: _,
                    } => args.clone(),
                    _ => {
                        return Err(TypeInferenceError::TypeEval(
                            TypeEvalError::CallNonFunction(func_type_sig),
                        ));
                    }
                };

                for (arg, param) in args.into_iter().zip(ctx[call].params.clone().into_iter()) {
                    let param_type = param
                        .eval_type(&mut self.symbols, ctx)
                        .map_err(TypeInferenceError::TypeEval)?;

                    self.add_constraint(arg, param_type)
                }
            }
            Expr::Identifier(_) => {}
            Expr::StructInit(st_init) => {
                let st_name = ctx[st_init].struct_name;
                let st = self
                    .symbols
                    .lookup(ctx, st_name)
                    .ok_or(TypeInferenceError::TypeEval(TypeEvalError::UnknownIdent(
                        st_name,
                    )))?
                    .unwrap_struct(ctx);

                for val in ctx[st_init].values.clone() {
                    let st_attr =
                        st.lookup_attr(ctx[val].name, ctx)
                            .ok_or(TypeInferenceError::TypeEval(TypeEvalError::UnknownIdent(
                                ctx[val].name,
                            )))?;

                    let val_type = ctx[val]
                        .value
                        .clone()
                        .eval_type(&mut self.symbols, ctx)
                        .map_err(TypeInferenceError::TypeEval)?;

                    self.add_constraint(ctx[st_attr].type_sig, val_type);
                }
            }
            Expr::StructAccess(_) => {}
            Expr::TupleAccess(_) => {}
            Expr::EscapeBlock(_) => {}
            Expr::Assignment(asg) => {
                let lhs = ctx[asg]
                    .lhs
                    .clone()
                    .eval_type(&mut self.symbols, ctx)
                    .map_err(TypeInferenceError::TypeEval)?;
                let rhs = ctx[asg]
                    .rhs
                    .clone()
                    .eval_type(&mut self.symbols, ctx)
                    .map_err(TypeInferenceError::TypeEval)?;
                self.add_constraint(lhs, rhs);
            }
            Expr::Tuple(_) => {}
            Expr::EnumInit(_) => {}
            Expr::UnresolvedMemberAccess(_) => {}
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::{
        node::type_signature::BuiltinType,
        test_utils::utils::{lowered_ir, type_infer},
    };

    use super::*;

    fn assert_type_mismatch<'a>(
        infer_result: Result<TypeInferrer<'a>, TypeInferenceError<'a>>,
        type_a: TypeSignature<'a>,
        type_b: TypeSignature<'a>,
    ) {
        match infer_result {
            Err(TypeInferenceError::ConflictingTypes(a, b)) => {
                assert!(type_a == a || type_a == b, "expected A did not match");
                assert!(type_b == a || type_b == b, "expected B did not match");
                assert!((a == type_a && b == type_b) || (a == type_b && b == type_a));
            }
            val => assert!(false, "expected conflicting type error, got {val:?}"),
        }
    }

    #[test]
    fn test_var_decl_mismatched_types() {
        let mut ir = lowered_ir("let x: String = 2").unwrap();

        assert_type_mismatch(
            type_infer(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::String),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
        );
    }

    #[test]
    fn test_var_decl_var() {
        let mut ir = lowered_ir("let a = true; let b: Boolean = a").unwrap();
        assert_matches!(type_infer(&mut ir), Ok(_));

        let mut ir = lowered_ir("let a = true; let b: Number = a").unwrap();
        assert_type_mismatch(
            type_infer(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
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
            type_infer(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
        );
    }

    #[test]
    fn test_struct_decl_attr_mismatched_types() {
        let mut ir = lowered_ir("struct Test { let attr: String = true }").unwrap();

        assert_type_mismatch(
            type_infer(&mut ir),
            ir.ctx.get_builtin_type_sig(BuiltinType::String),
            ir.ctx.get_builtin_type_sig(BuiltinType::Boolean),
        );
    }

    #[test]
    fn test_call_non_function() {
        let mut ir = lowered_ir("let val = true; val()").unwrap();

        match type_infer(&mut ir) {
            Err(TypeInferenceError::TypeEval(TypeEvalError::CallNonFunction(expr_type))) => {
                assert_eq!(expr_type, ir.ctx.get_builtin_type_sig(BuiltinType::Boolean))
            }
            _ => assert!(false),
        }
    }
}
