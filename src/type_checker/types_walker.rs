use id_arena::Id;

use crate::{
    ir::{
        ast_walker::{AstWalker, ScopeValue},
        context::IrCtx,
        node::{
            expression::Expr,
            statement::Stmt,
            type_signature::{BuiltinType, TypeSignature, Typed},
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

impl<'a> AstWalker<'a> for TypeChecker<'a> {
    type Error = TypeCheckerError<'a>;

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match value {
            ScopeValue::Func(func) => {
                self.symbols
                    .enter_scope(func.name.clone())
                    .expect("scope should exist");
            }
            ScopeValue::Struct(st) => {
                self.symbols
                    .enter_scope(st.name.clone())
                    .expect("scope should exist");
            }
            ScopeValue::StructInit(st_init) => self
                .symbols
                .enter_scope(st_init.scope_name.clone())
                .expect("scope should exist"),
            ScopeValue::Enum(enm) => self
                .symbols
                .enter_scope(enm.name.clone())
                .expect("scope should exist"),
        }

        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        child: Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        self.symbols
            .exit_scope()
            .expect("scope should not be global scope");

        Ok(())
    }

    fn pre_visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        stmt: Id<Stmt<'a>>,
    ) -> Result<(), Self::Error> {
        match ctx.nodes.stmts[stmt] {
            Stmt::VariableDecl(var_decl) => match &mut var_decl.value {
                Expr::Function(func) => match &var_decl.type_sig {
                    Some(
                        type_sig @ TypeSignature::Function {
                            args: _,
                            return_type: _,
                        },
                    ) => {
                        fill_type_signature(ctx, &mut self.symbols, func, Some(type_sig.clone()))?;
                    }
                    Some(type_sig) => {
                        return Err(TypeCheckerError::TypeSignatureMismatch {
                            type_sig,
                            expr_type: TypeSignature::Function {
                                args: vec![],
                                return_type: ctx.get_builtin_type_sig(BuiltinType::Void),
                            },
                        })
                    }
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
        stmt: Id<Stmt<'a>>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match stmt {
            Stmt::VariableDecl(var_decl) => {
                self.symbols.visit_next_symbol();
                type_check(ctx, &mut self.symbols, var_decl)
            }
            Stmt::FunctionDecl(func_decl) => type_check(ctx, &mut self.symbols, func_decl),
            Stmt::StructDecl(st) => {
                for attr in &mut st.attrs {
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
        _scope: &mut (),
        expr: &mut Expr<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match expr {
            Expr::FunctionCall(call) => {
                match call
                    .func
                    .eval_type(&mut self.symbols)
                    .map_err(TypeCheckerError::TypeEvalError)?
                {
                    TypeSignature::Function { args, return_type } => {
                        let param_types = call
                            .params
                            .iter()
                            .map(|param| param.eval_type(&mut self.symbols).unwrap())
                            .collect::<Vec<_>>();

                        let arg_count_match = call.params.len() == args.len();
                        let args_match = param_types.iter().zip(args.iter()).all(|(a, b)| *a == *b);

                        if !arg_count_match || !args_match {
                            return Err(TypeCheckerError::TypeSignatureMismatch {
                                type_sig: TypeSignature::Function {
                                    args,
                                    return_type: return_type.clone(),
                                },
                                expr_type: TypeSignature::Function {
                                    args: param_types,
                                    return_type,
                                },
                            });
                        }

                        Ok(())
                    }
                    type_sig => Err(TypeCheckerError::CallNonFunction {
                        ident_type: type_sig,
                    }),
                }
            }
            Expr::Function(func) => type_check(&mut self.symbols, func),
            Expr::Assignment(asg) => check_assignment(&mut self.symbols, asg),
            Expr::StructInit(st_init) => check_struct_init(&mut self.symbols, st_init),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ir::test_utils::utils::type_check, parser::parse_ast, symbols::builtin_types::BuiltinType,
        type_checker::TypeCheckerError,
    };

    #[test]
    fn test_var_decl_matching_types() {
        let mut ast = parse_ast("let x: String = \"hello\"").unwrap();
        assert!(type_check(&mut ast).is_ok());
    }

    #[test]
    fn test_var_decl_mismatched_types() {
        let mut ast = parse_ast("let x: String = 2").unwrap();

        match type_check(&mut ast) {
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig,
                expr_type,
            }) => {
                assert_eq!(type_sig, BuiltinType::String.type_sig());
                assert_eq!(expr_type, BuiltinType::Number.type_sig())
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_struct_decl_attr_mismatched_types() {
        let mut ast = parse_ast("struct Test { let attr: String = true }").unwrap();

        match type_check(&mut ast) {
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig,
                expr_type,
            }) => {
                assert_eq!(type_sig, BuiltinType::String.type_sig());
                assert_eq!(expr_type, BuiltinType::Boolean.type_sig())
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_struct_access_mismatched_types() {
        let mut ast = parse_ast(
            "\
        struct Test { let attr: Number }
        let test = Test { attr: 123 }
        let wrong: Boolean = test.attr",
        )
        .unwrap();

        match type_check(&mut ast) {
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig,
                expr_type,
            }) => {
                assert_eq!(type_sig, BuiltinType::Boolean.type_sig());
                assert_eq!(expr_type, BuiltinType::Number.type_sig());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_struct_init_default() {
        let mut ast = parse_ast(
            "\
        struct Test { let default = 34; let noDefault: Number }
        let test = Test { noDefault: 123 }",
        )
        .unwrap();

        assert!(type_check(&mut ast).is_ok())
    }

    #[test]
    fn test_struct_init_not_default() {
        let mut ast = parse_ast(
            "\
        struct Test { let noDefault: Number }
        let test = Test {}",
        )
        .unwrap();

        match type_check(&mut ast) {
            Err(_) => {}
            _ => assert!(false),
        }
    }

    #[test]
    fn test_var_assign_var() {
        let mut ast = parse_ast("let a = true; let b: Boolean = a").unwrap();
        assert_matches!(type_check(&mut ast), Ok(_));

        let mut ast = parse_ast("let a = true; let b: Number = a").unwrap();
        match type_check(&mut ast) {
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig,
                expr_type,
            }) => {
                assert_eq!(type_sig, BuiltinType::Number.type_sig());
                assert_eq!(expr_type, BuiltinType::Boolean.type_sig());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_call_non_function() {
        let mut ast = parse_ast("let val = true; val()").unwrap();

        match type_check(&mut ast) {
            Err(TypeCheckerError::CallNonFunction { ident_type }) => {
                assert_eq!(ident_type, BuiltinType::Boolean.type_sig())
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_escape_block_function_return() {
        let mut ast = parse_ast("func f() -> Number { return @{ 1 + 2 } }").unwrap();
        assert_matches!(type_check(&mut ast), Ok(_));

        let mut ast = parse_ast("func f() -> Number { return @{ 1 + 2 }; return 2 }").unwrap();
        assert_matches!(type_check(&mut ast), Ok(_));
    }
}
