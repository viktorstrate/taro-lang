use std::collections::HashMap;

use crate::ast::nodes::{
    identifier::{Ident, Identifiable},
    statements::VarDecl,
};

pub mod symbol_walker;

#[derive(Default, Debug)]
pub struct SymbolTable<'a> {
    table: HashMap<Ident<'a>, VarDecl<'a>>,
    scopes: HashMap<Ident<'a>, SymbolTable<'a>>,
}

#[derive(Debug)]
pub enum SymbolsError<'a> {
    SymbolAlreadyExistsInScope(Ident<'a>),
}

impl<'a> SymbolTable<'a> {
    pub fn insert(&mut self, val: VarDecl<'a>) -> Result<&mut VarDecl<'a>, SymbolsError<'a>> {
        let key: Ident<'a> = val.name().clone();
        let error_ident = key.clone();

        self.table
            .try_insert(key, val)
            .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(error_ident))
    }

    pub fn insert_scope(
        &mut self,
        ident: Ident<'a>,
        scope: SymbolTable<'a>,
    ) -> Result<&mut SymbolTable<'a>, SymbolsError<'a>> {
        let error_ident = ident.clone();

        self.scopes
            .try_insert(ident, scope)
            .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(error_ident))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{ast::ast_walker::walk_ast, parser::parse_ast};

    use super::symbol_walker::SymbolCollector;

    #[test]
    fn test_existing_symbol_error() {
        let ast = parse_ast("let x = 2; let x = 3").unwrap();
        let mut collector = SymbolCollector::default();
        let result = walk_ast(&mut collector, &ast);
        assert_matches!(result, Err(_));
    }
}
