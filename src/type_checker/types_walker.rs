use crate::{
    ir::{
        context::IrCtx,
        ir_walker::{IrWalker, ScopeValue},
        node::{
            expression::Expr,
            statement::Stmt,
            type_signature::{BuiltinType, TypeSignatureValue, Typed},
            NodeRef,
        },
    },
    symbols::{symbol_table::symbol_table_zipper::SymbolTableZipper, symbol_table::SymbolTable},
};

use super::{
    assignment::check_assignment,
    struct_type::check_struct_init,
    types_helpers::{fill_type_signature, type_check},
    TypeCheckerError,
};

pub struct TypeChecker<'a> {
    pub symbols: SymbolTableZipper<'a>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(symbols: SymbolTable<'a>) -> Self {
        TypeChecker {
            symbols: symbols.into(),
        }
    }
}

impl<'a> IrWalker<'a> for TypeChecker<'a> {
    type Error = TypeCheckerError<'a>;

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match value {
            ScopeValue::Func(func) => {
                let name = ctx[func].name;
                self.symbols
                    .enter_scope(ctx, name)
                    .expect("scope should exist");
            }
            ScopeValue::Struct(st) => {
                let name = ctx[st].name;
                self.symbols
                    .enter_scope(ctx, name)
                    .expect("scope should exist");
            }
            ScopeValue::StructInit(st_init) => {
                let scope_name = ctx[st_init].scope_name;
                self.symbols
                    .enter_scope(ctx, scope_name)
                    .expect("scope should exist")
            }
            ScopeValue::Enum(enm) => {
                let name = ctx[enm].name;
                self.symbols
                    .enter_scope(ctx, name)
                    .expect("scope should exist")
            }
        }

        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        _child: Self::Scope,
        _value: ScopeValue<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        self.symbols
            .exit_scope(ctx)
            .expect("scope should not be global scope");

        Ok(())
    }

    fn pre_visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        stmt: NodeRef<'a, Stmt<'a>>,
    ) -> Result<(), Self::Error> {
        match ctx[stmt] {
            Stmt::VariableDecl(var_decl) => match ctx[ctx[var_decl].value] {
                Expr::Function(func) => match ctx[var_decl].type_sig {
                    Some(type_sig) => match &ctx[type_sig] {
                        TypeSignatureValue::Function {
                            args: _,
                            return_type: _,
                        } => {
                            fill_type_signature(ctx, &mut self.symbols, func, Some(type_sig))?;
                        }
                        _ => {
                            return Err(TypeCheckerError::TypeSignatureMismatch {
                                type_sig,
                                expr_type: ctx.get_type_sig(TypeSignatureValue::Function {
                                    args: vec![],
                                    return_type: ctx.get_builtin_type_sig(BuiltinType::Void),
                                }),
                            })
                        }
                    },
                    None => {}
                },
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    fn visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        stmt: NodeRef<'a, Stmt<'a>>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match ctx[stmt].clone() {
            Stmt::VariableDecl(var_decl) => {
                self.symbols.visit_next_symbol(ctx);
                type_check(ctx, &mut self.symbols, var_decl)
            }
            Stmt::FunctionDecl(func_decl) => type_check(ctx, &mut self.symbols, func_decl),
            Stmt::StructDecl(st) => {
                for attr in ctx[st].attrs.clone() {
                    type_check(ctx, &mut self.symbols, attr)?;
                }
                Ok(())
            }
            Stmt::EnumDecl(enm) => {
                type_check(ctx, &mut self.symbols, enm)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut (),
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match ctx[expr].clone() {
            Expr::FunctionCall(call) => {
                let type_sig = ctx[call]
                    .func
                    .clone()
                    .eval_type(&mut self.symbols, ctx)
                    .map_err(TypeCheckerError::TypeEvalError)?;

                let (args, return_type) = match &ctx[type_sig] {
                    TypeSignatureValue::Function { args, return_type } => {
                        Ok((args.clone(), *return_type))
                    }
                    _ => Err(TypeCheckerError::CallNonFunction {
                        ident_type: type_sig,
                    }),
                }?;

                let param_types = ctx[call]
                    .params
                    .clone()
                    .into_iter()
                    .map(|param| param.eval_type(&mut self.symbols, ctx).unwrap())
                    .collect::<Vec<_>>();

                let arg_count_match = ctx[call].params.len() == args.len();
                let args_match = param_types.iter().zip(args.iter()).all(|(a, b)| *a == *b);

                if !arg_count_match || !args_match {
                    return Err(TypeCheckerError::TypeSignatureMismatch {
                        type_sig: ctx
                            .get_type_sig(TypeSignatureValue::Function { args, return_type }),
                        expr_type: ctx.get_type_sig(TypeSignatureValue::Function {
                            args: param_types,
                            return_type,
                        }),
                    });
                }

                Ok(())
            }
            Expr::Function(func) => type_check(ctx, &mut self.symbols, func),
            Expr::Assignment(asg) => check_assignment(ctx, &mut self.symbols, asg),
            Expr::StructInit(st_init) => check_struct_init(ctx, &mut self.symbols, st_init),
            _ => Ok(()),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use crate::{
//         ir::test_utils::utils::type_check, parser::parse_ast, symbols::builtin_types::BuiltinType,
//         type_checker::TypeCheckerError,
//     };

//     #[test]
//     fn test_var_decl_matching_types() {
//         let mut ast = parse_ast("let x: String = \"hello\"").unwrap();
//         assert!(type_check(&mut ast).is_ok());
//     }

//     #[test]
//     fn test_var_decl_mismatched_types() {
//         let mut ast = parse_ast("let x: String = 2").unwrap();

//         match type_check(&mut ast) {
//             Err(TypeCheckerError::TypeSignatureMismatch {
//                 type_sig,
//                 expr_type,
//             }) => {
//                 assert_eq!(type_sig, BuiltinType::String.type_sig());
//                 assert_eq!(expr_type, BuiltinType::Number.type_sig())
//             }
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_struct_decl_attr_mismatched_types() {
//         let mut ast = parse_ast("struct Test { let attr: String = true }").unwrap();

//         match type_check(&mut ast) {
//             Err(TypeCheckerError::TypeSignatureMismatch {
//                 type_sig,
//                 expr_type,
//             }) => {
//                 assert_eq!(type_sig, BuiltinType::String.type_sig());
//                 assert_eq!(expr_type, BuiltinType::Boolean.type_sig())
//             }
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_struct_access_mismatched_types() {
//         let mut ast = parse_ast(
//             "\
//         struct Test { let attr: Number }
//         let test = Test { attr: 123 }
//         let wrong: Boolean = test.attr",
//         )
//         .unwrap();

//         match type_check(&mut ast) {
//             Err(TypeCheckerError::TypeSignatureMismatch {
//                 type_sig,
//                 expr_type,
//             }) => {
//                 assert_eq!(type_sig, BuiltinType::Boolean.type_sig());
//                 assert_eq!(expr_type, BuiltinType::Number.type_sig());
//             }
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_struct_init_default() {
//         let mut ast = parse_ast(
//             "\
//         struct Test { let default = 34; let noDefault: Number }
//         let test = Test { noDefault: 123 }",
//         )
//         .unwrap();

//         assert!(type_check(&mut ast).is_ok())
//     }

//     #[test]
//     fn test_struct_init_not_default() {
//         let mut ast = parse_ast(
//             "\
//         struct Test { let noDefault: Number }
//         let test = Test {}",
//         )
//         .unwrap();

//         match type_check(&mut ast) {
//             Err(_) => {}
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_var_assign_var() {
//         let mut ast = parse_ast("let a = true; let b: Boolean = a").unwrap();
//         assert_matches!(type_check(&mut ast), Ok(_));

//         let mut ast = parse_ast("let a = true; let b: Number = a").unwrap();
//         match type_check(&mut ast) {
//             Err(TypeCheckerError::TypeSignatureMismatch {
//                 type_sig,
//                 expr_type,
//             }) => {
//                 assert_eq!(type_sig, BuiltinType::Number.type_sig());
//                 assert_eq!(expr_type, BuiltinType::Boolean.type_sig());
//             }
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_call_non_function() {
//         let mut ast = parse_ast("let val = true; val()").unwrap();

//         match type_check(&mut ast) {
//             Err(TypeCheckerError::CallNonFunction { ident_type }) => {
//                 assert_eq!(ident_type, BuiltinType::Boolean.type_sig())
//             }
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_escape_block_function_return() {
//         let mut ast = parse_ast("func f() -> Number { return @{ 1 + 2 } }").unwrap();
//         assert_matches!(type_check(&mut ast), Ok(_));

//         let mut ast = parse_ast("func f() -> Number { return @{ 1 + 2 }; return 2 }").unwrap();
//         assert_matches!(type_check(&mut ast), Ok(_));
//     }
// }
