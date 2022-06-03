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

    fn visit_declaration(
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
    fn test_declaration_matching_types() {
        let ast = parse_ast("let x: String = \"hello\"").unwrap();
        let mut collector = TypeChecker {
            symbols: SymbolTable::default(),
        };
        let result = walk_ast(&mut collector, &ast);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_declaration_mismatched_types() {
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
}
