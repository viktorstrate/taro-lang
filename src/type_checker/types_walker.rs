use crate::{
    ast::{
        ast_walker::{AstWalker, ScopeValue},
        node::{
            expression::Expr,
            function::Function,
            statement::Stmt,
            structure::Struct,
            type_signature::{TypeSignature, Typed},
        },
    },
    symbols::{symbol_table::SymbolTable, symbol_table_zipper::SymbolTableZipper},
};

use super::{specialize_type::specialize_type, TypeCheckerError};

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
        _parent: &mut (),
        value: ScopeValue<'a, '_>,
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
        }

        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        _parent: &mut (),
        _child: (),
        _value: ScopeValue<'a, '_>,
    ) -> Result<(), TypeCheckerError<'a>> {
        self.symbols
            .exit_scope()
            .expect("scope should not be global scope");

        Ok(())
    }

    fn visit_stmt(
        &mut self,
        _scope: &mut (),
        stmt: &mut Stmt<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match stmt {
            Stmt::VariableDecl(var_decl) => {
                let val_type = var_decl
                    .value
                    .type_sig(&mut self.symbols)
                    .map_err(TypeCheckerError::ValueError)?;

                // specialize specified type signature
                match var_decl.type_sig.as_mut() {
                    Some(mut type_sig) => specialize_type(&mut self.symbols, &mut type_sig)?,
                    None => {}
                };

                if let Some(type_sig) = &var_decl.type_sig {
                    // make sure specified type matches expression
                    if val_type != *type_sig {
                        return Err(TypeCheckerError::TypeSignatureMismatch::<'a> {
                            type_sig: type_sig.clone(),
                            expr_type: val_type,
                        });
                    }
                } else {
                    // set declaration type to the calculated type of the expression
                    var_decl.type_sig = Some(val_type);
                }

                Ok(())
            }
            Stmt::FunctionDecl(func_decl) => {
                // specialize specified return type
                match func_decl.return_type.as_mut() {
                    Some(mut type_sig) => specialize_type(&mut self.symbols, &mut type_sig)?,
                    None => {}
                };

                let func_type = func_decl
                    .type_sig(&mut self.symbols)
                    .map_err(TypeCheckerError::FunctionError)?;

                let body_type = match func_type {
                    TypeSignature::Function {
                        args: _,
                        return_type,
                    } => *return_type,
                    _ => unreachable!(),
                };

                if let Some(return_type) = &func_decl.return_type {
                    // make sure the specified return type matches the actual return type
                    if body_type != *return_type {
                        return Err(TypeCheckerError::TypeSignatureMismatch::<'a> {
                            type_sig: return_type.clone(),
                            expr_type: body_type,
                        });
                    }
                } else {
                    // set return type to the calculated type of the function body
                    func_decl.return_type = Some(body_type);
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn visit_struct_decl(
        &mut self,
        _scope: &mut (),
        st: &mut Struct<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        for attr in &st.attrs {
            match (&attr.type_sig, &attr.default_value) {
                (Some(type_sig), Some(val)) => {
                    let val_type = val
                        .type_sig(&mut self.symbols)
                        .map_err(TypeCheckerError::ValueError)?;
                    if *type_sig != val_type {
                        return Err(TypeCheckerError::TypeSignatureMismatch::<'a> {
                            type_sig: type_sig.clone(),
                            expr_type: val_type,
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn visit_expr(&mut self, expr: &mut Expr<'a>) -> Result<(), TypeCheckerError<'a>> {
        match expr {
            Expr::FunctionCall(call) => {
                match call
                    .func
                    .type_sig(&mut self.symbols)
                    .map_err(TypeCheckerError::ValueError)?
                {
                    TypeSignature::Function { args, return_type } => {
                        let param_types = call
                            .params
                            .iter()
                            .map(|param| param.type_sig(&mut self.symbols).unwrap())
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
            Expr::Function(func) => {
                // specialize specified return type
                match func.return_type.as_mut() {
                    Some(mut return_type) => {
                        specialize_type(&mut self.symbols, &mut return_type)?;
                    }
                    None => {}
                };

                if let Some(return_sig) = &func.return_type {
                    let func_type = func
                        .type_sig(&mut self.symbols)
                        .map_err(TypeCheckerError::FunctionError)?;

                    // get function return type
                    let body_type = match func_type {
                        TypeSignature::Function {
                            args: _,
                            return_type,
                        } => *return_type,
                        _ => unreachable!(),
                    };

                    if body_type != *return_sig {
                        return Err(TypeCheckerError::TypeSignatureMismatch {
                            type_sig: return_sig.clone(),
                            expr_type: body_type,
                        });
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::test_utils::utils::type_check, parser::parse_ast, symbols::builtin_types::BuiltinType,
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
                assert_eq!(expr_type, BuiltinType::Bool.type_sig())
            }
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
                assert_eq!(expr_type, BuiltinType::Bool.type_sig());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_call_non_function() {
        let mut ast = parse_ast("let val = true; val()").unwrap();

        match type_check(&mut ast) {
            Err(TypeCheckerError::CallNonFunction { ident_type }) => {
                assert_eq!(ident_type, BuiltinType::Bool.type_sig())
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_decl_inside_scope() {
        let mut ast = parse_ast("let f = () -> Boolean { let a = true; return a }").unwrap();
        assert_matches!(type_check(&mut ast), Ok(_))
    }
}
