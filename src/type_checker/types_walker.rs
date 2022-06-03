use crate::{
    ast::{ast_walker::AstWalker, nodes::statements::VarDecl},
    symbols::SymbolTable,
};

use super::TypeCheckerError;

pub struct TypeChecker<'a> {
    symbols: SymbolTable<'a>,
}

impl<'a> AstWalker<'a> for TypeChecker<'a> {
    type Error = TypeCheckerError<'a>;

    fn visit_var_decl(
        &mut self,
        scope: &mut Self::Scope,
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

    fn visit_struct_decl(
        &mut self,
        st: &crate::ast::nodes::structures::Struct<'a>,
    ) -> Result<(), Self::Error> {
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
        ast::{ast_walker::walk_ast, nodes::type_signature::BuiltinType},
        parser::parse_ast,
        symbols::SymbolTable,
        type_checker::{types_walker::TypeChecker, TypeCheckerError},
    };

    #[test]
    fn test_var_decl_matching_types() {
        let ast = parse_ast("let x: String = \"hello\"").unwrap();
        let mut checker = TypeChecker {
            symbols: SymbolTable::default(),
        };
        let result = walk_ast(&mut checker, &ast);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_var_decl_mismatched_types() {
        let ast = parse_ast("let x: String = 2").unwrap();
        let mut collector = TypeChecker {
            symbols: SymbolTable::default(),
        };
        let result = walk_ast(&mut collector, &ast);
        assert_eq!(
            result,
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig: BuiltinType::String.into(),
                expr_type: BuiltinType::Number.into()
            })
        );
    }

    #[test]
    fn test_struct_decl_attr_mismatched_types() {
        let ast = parse_ast("struct Test { let attr: String = true }").unwrap();
        let mut checker = TypeChecker {
            symbols: SymbolTable::default(),
        };
        let result = walk_ast(&mut checker, &ast);
        assert_eq!(
            result,
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig: BuiltinType::String.into(),
                expr_type: BuiltinType::Bool.into()
            })
        );
    }
}
