use crate::{
    ast::{
        ast_walker::AstWalker,
        node::{statement::VarDecl, structure::Struct},
    },
    symbols::{SymbolTable, SymbolTableZipper},
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
        decl: &VarDecl<'a>,
    ) -> Result<(), Self::Error> {
        let Some(type_sig) = &decl.type_sig else {
            return Ok(());
        };

        let val_type = decl.value.value_type();

        if val_type != *type_sig {
            return Err(TypeCheckerError::TypeSignatureMismatch::<'a> {
                type_sig: type_sig.clone(),
                expr_type: val_type,
            });
        }

        Ok(())
    }

    fn visit_struct_decl(&mut self, st: &Struct<'a>) -> Result<(), Self::Error> {
        for attr in &st.attrs {
            match (&attr.type_sig, &attr.default_value) {
                (Some(type_sig), Some(val)) => {
                    let val_type = val.value_type();
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
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{ast_walker::walk_ast, node::type_signature::BuiltinType, AST},
        parser::parse_ast,
        symbols::symbol_walker::SymbolCollector,
        type_checker::{types_walker::TypeChecker, TypeCheckerError},
    };

    #[test]
    fn test_var_decl_matching_types() {
        let ast = parse_ast("let x: String = \"hello\"").unwrap();
        assert_eq!(type_check(&ast), Ok(()));
    }

    #[test]
    fn test_var_decl_mismatched_types() {
        let ast = parse_ast("let x: String = 2").unwrap();

        assert_eq!(
            type_check(&ast),
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig: BuiltinType::String.into(),
                expr_type: BuiltinType::Number.into()
            })
        );
    }

    #[test]
    fn test_struct_decl_attr_mismatched_types() {
        let ast = parse_ast("struct Test { let attr: String = true }").unwrap();

        assert_eq!(
            type_check(&ast),
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig: BuiltinType::String.into(),
                expr_type: BuiltinType::Bool.into()
            })
        );
    }

    #[test]
    fn test_call_non_function() {
        let ast = parse_ast("let val = true; val()").unwrap();
        assert_eq!(
            type_check(&ast),
            Err(TypeCheckerError::CallNonFunction {
                ident_type: BuiltinType::Bool.into()
            })
        )
    }

    fn type_check<'a>(ast: &AST<'a>) -> Result<(), TypeCheckerError<'a>> {
        let mut sym_collector = SymbolCollector {};
        let symbols = walk_ast(&mut sym_collector, &ast).unwrap();

        let mut checker = TypeChecker::new(symbols);
        return walk_ast(&mut checker, &ast);
    }
}
