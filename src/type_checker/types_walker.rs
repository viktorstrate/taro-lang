use crate::{
    ast::{
        ast_walker::AstWalker,
        node::{
            expression::Expr,
            statement::Stmt,
            structure::Struct,
            type_signature::{TypeSignature, Typed},
        },
    },
    symbols::{symbol_table::SymbolTable, symbol_table_zipper::SymbolTableZipper},
};

use super::{function_type::func_body_type_sig, TypeCheckerError};

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
        _parent: &mut Self::Scope,
        scope_ident: &crate::ast::node::identifier::Ident<'a>,
    ) -> Result<Self::Scope, Self::Error> {
        self.symbols
            .enter_scope(scope_ident.clone())
            .expect("scope should exist");

        println!("ENTER SCOPE {:?}", scope_ident);

        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        _parent: &mut Self::Scope,
        _child: Self::Scope,
        scope_ident: &crate::ast::node::identifier::Ident<'a>,
    ) -> Result<(), Self::Error> {
        self.symbols
            .exit_scope()
            .expect("scope should not be global scope");

        println!("EXIT SCOPE {:?}", scope_ident);

        Ok(())
    }

    fn visit_stmt(
        &mut self,
        _scope: &mut Self::Scope,
        stmt: &mut Stmt<'a>,
    ) -> Result<(), Self::Error> {
        match stmt {
            Stmt::VariableDecl(var_decl) => {
                let val_type = var_decl
                    .value
                    .type_sig(&self.symbols)
                    .map_err(TypeCheckerError::ValueError)?;

                if let Some(type_sig) = &var_decl.type_sig {
                    // make sure specified type matches expression
                    if val_type != *type_sig {
                        return Err(TypeCheckerError::TypeSignatureMismatch::<'a> {
                            type_sig: type_sig.clone(),
                            expr_type: val_type,
                        });
                    }
                } else {
                    // set declaration type to the type of the expression
                    var_decl.type_sig = Some(val_type);
                }

                Ok(())
            }
            Stmt::FunctionDecl(_func_decl) => {
                // TODO: Type check function declerations
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn visit_struct_decl(&mut self, st: &mut Struct<'a>) -> Result<(), Self::Error> {
        for attr in &st.attrs {
            match (&attr.type_sig, &attr.default_value) {
                (Some(type_sig), Some(val)) => {
                    let val_type = val
                        .type_sig(&self.symbols)
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

    fn visit_expr(&mut self, expr: &mut Expr<'a>) -> Result<(), Self::Error> {
        match expr {
            Expr::FunctionCall(call) => {
                match call
                    .func
                    .type_sig(&self.symbols)
                    .map_err(TypeCheckerError::ValueError)?
                {
                    TypeSignature::Function { args, return_type } => {
                        let param_types = call
                            .params
                            .iter()
                            .map(|param| param.type_sig(&self.symbols).unwrap());
                        let arg_count_match = call.params.len() == args.len();

                        let args_match = param_types.clone().zip(args.iter()).all(|(a, b)| a == *b);
                        if !arg_count_match || !args_match {
                            return Err(TypeCheckerError::TypeSignatureMismatch {
                                type_sig: TypeSignature::Function {
                                    args,
                                    return_type: return_type.clone(),
                                },
                                expr_type: TypeSignature::Function {
                                    args: Box::new(param_types.collect()),
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
                if let Some(return_sig) = &func.return_type {
                    let body_type = func_body_type_sig(&self.symbols, func)
                        .map_err(TypeCheckerError::FunctionError)?;

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
        ast::{node::type_signature::BuiltinType, test_utils::utils::type_check},
        parser::parse_ast,
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
                assert_eq!(type_sig, BuiltinType::String.into());
                assert_eq!(expr_type, BuiltinType::Number.into())
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
                assert_eq!(type_sig, BuiltinType::String.into());
                assert_eq!(expr_type, BuiltinType::Bool.into())
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
                assert_eq!(type_sig, BuiltinType::Number.into());
                assert_eq!(expr_type, BuiltinType::Bool.into());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_call_non_function() {
        let mut ast = parse_ast("let val = true; val()").unwrap();

        match type_check(&mut ast) {
            Err(TypeCheckerError::CallNonFunction { ident_type }) => {
                assert_eq!(ident_type, BuiltinType::Bool.into())
            }
            _ => assert!(false),
        }
    }
}
