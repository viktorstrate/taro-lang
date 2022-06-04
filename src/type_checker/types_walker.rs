use crate::{
    ast::{
        ast_walker::AstWalker,
        node::{statement::VarDecl, structure::Struct, type_signature::TypeSignature},
    },
    symbols::{symbol_table::SymbolTable, symbol_table_zipper::SymbolTableZipper},
};

use super::TypeCheckerError;

pub struct TypeChecker<'a> {
    symbols: SymbolTableZipper<'a>,
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

        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        _parent: &mut Self::Scope,
        _child: Self::Scope,
        _scope_ident: &crate::ast::node::identifier::Ident<'a>,
    ) -> Result<(), Self::Error> {
        self.symbols
            .exit_scope()
            .expect("scope should not be global scope");

        Ok(())
    }

    fn visit_var_decl(
        &mut self,
        _scope: &mut Self::Scope,
        decl: &mut VarDecl<'a>,
    ) -> Result<(), Self::Error> {
        let Some(type_sig) = &decl.type_sig else {
            return Ok(());
        };

        let val_type = decl
            .value
            .value_type(&self.symbols)
            .map_err(TypeCheckerError::ValueError)?;

        if val_type != *type_sig {
            return Err(TypeCheckerError::TypeSignatureMismatch::<'a> {
                type_sig: type_sig.clone(),
                expr_type: val_type,
            });
        }

        Ok(())
    }

    fn visit_struct_decl(&mut self, st: &mut Struct<'a>) -> Result<(), Self::Error> {
        for attr in &st.attrs {
            match (&attr.type_sig, &attr.default_value) {
                (Some(type_sig), Some(val)) => {
                    let val_type = val
                        .value_type(&self.symbols)
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

    fn visit_expr(
        &mut self,
        expr: &mut crate::ast::node::expression::Expr<'a>,
    ) -> Result<(), Self::Error> {
        match expr {
            crate::ast::node::expression::Expr::FunctionCall(call) => {
                match call
                    .func
                    .value_type(&self.symbols)
                    .map_err(TypeCheckerError::ValueError)?
                {
                    TypeSignature::Function { args, return_type } => {
                        let param_types = call
                            .params
                            .iter()
                            .map(|param| param.value_type(&self.symbols).unwrap());
                        let arg_count_match = call.params.len() != args.len();

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
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::assert_matches::assert_matches;

    use crate::{
        ast::{ast_walker::walk_ast, node::type_signature::BuiltinType, AST},
        parser::parse_ast,
        symbols::symbol_walker::SymbolCollector,
        type_checker::{types_walker::TypeChecker, TypeCheckerError},
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

    fn type_check<'a>(ast: &mut AST<'a>) -> Result<(), TypeCheckerError<'a>> {
        let mut sym_collector = SymbolCollector {};
        let symbols = walk_ast(&mut sym_collector, ast).unwrap();

        let mut checker = TypeChecker::new(symbols);
        return walk_ast(&mut checker, ast);
    }
}
