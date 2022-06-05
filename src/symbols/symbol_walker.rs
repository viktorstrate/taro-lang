use crate::ast::{
    ast_walker::AstWalker,
    node::{
        identifier::Ident,
        statement::Stmt::{self, FunctionDecl, VariableDecl},
    },
};

use super::symbol_table::{SymbolTable, SymbolsError};

#[derive(Default)]
pub struct SymbolCollector {}

impl<'a> AstWalker<'a> for SymbolCollector {
    type Scope = SymbolTable<'a>;
    type Error = SymbolsError<'a>;

    fn visit_stmt(
        &mut self,
        scope: &mut Self::Scope,
        stmt: &mut Stmt<'a>,
    ) -> Result<(), Self::Error> {
        match stmt {
            VariableDecl(var_decl) => scope.insert(var_decl.clone().into()).map(|_| ()),
            FunctionDecl(func_decl) => scope.insert(func_decl.clone().into()).map(|_| ()),
            _ => Ok(()),
        }
    }

    fn visit_scope_end(
        &mut self,
        parent: &mut Self::Scope,
        child: Self::Scope,
        scope_ident: &Ident<'a>,
    ) -> Result<(), Self::Error> {
        // save child scope in parent scope
        parent.insert_scope(scope_ident.clone(), child).map(|_| ())
    }

    fn visit_scope_begin(
        &mut self,
        _parent: &mut Self::Scope,
        _scope_ident: &Ident<'a>,
    ) -> Result<Self::Scope, Self::Error> {
        Ok(Self::Scope::default())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::{
            ast_walker::walk_ast,
            node::{
                expression::Expr,
                identifier::{Ident, Identifiable},
            },
        },
        parser::parse_ast,
        symbols::{symbol_table::SymbolValue, symbol_walker::SymbolCollector},
    };

    #[test]
    fn test_existing_symbol_error() {
        let mut ast = parse_ast("let x = 2; let x = 3").unwrap();
        let mut collector = SymbolCollector::default();
        let result = walk_ast(&mut collector, &mut ast);
        assert_matches!(result, Err(_));
    }

    #[test]
    fn test_locate_symbol() {
        let mut ast = parse_ast("let x: Boolean = true").unwrap();
        let mut collector = SymbolCollector::default();
        let symtable = walk_ast(&mut collector, &mut ast).unwrap();
        let sym_val = symtable.locate(&Ident::new_unplaced("x")).unwrap();

        assert_eq!(*sym_val.name(), Ident::new_unplaced("x"));
        match sym_val {
            SymbolValue::VarDecl(var_decl) => {
                assert_matches!(var_decl.value, Expr::BoolLiteral(true));
            }
            _ => assert!(false),
        }
    }
}
