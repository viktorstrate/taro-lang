use std::collections::HashMap;

use crate::{
    ir::{
        context::IrCtx,
        ir_walker::IrWalker,
        node::{
            expression::Expr,
            function::Function,
            statement::Stmt,
            type_signature::{TypeEvalError, TypeSignature, TypeSignatureValue, Typed},
            NodeRef,
        },
    },
    symbols::{
        symbol_resolver::SymbolResolver, symbol_table::symbol_table_zipper::SymbolTableZipper,
    },
};

use super::{
    coercion::{can_coerce_to, coerce},
    types_helpers::types_match,
};

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

    fn visit_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        while let Some(TypeConstraint(type_a, type_b)) = self.constraints.pop() {
            match (ctx[type_a].clone(), ctx[type_b].clone()) {
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
