use std::collections::HashMap;

use crate::ast::node::{
    identifier::{Ident, Identifiable},
    statement::VarDecl,
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
    ScopeNotFound(Ident<'a>),
    MovePastGlobelScope,
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

    pub fn remove_scope(&mut self, ident: &Ident<'a>) -> Option<SymbolTable<'a>> {
        self.scopes.remove(ident)
    }

    pub fn locate(&self, ident: &Ident<'a>) -> Option<&VarDecl<'a>> {
        self.table.get(ident)
    }
}

impl<'a> Into<SymbolTableZipper<'a>> for SymbolTable<'a> {
    fn into(self) -> SymbolTableZipper<'a> {
        SymbolTableZipper {
            cursor: self,
            breadcrumb: Vec::new(),
        }
    }
}

pub struct SymbolTableZipper<'a> {
    cursor: SymbolTable<'a>,
    breadcrumb: Vec<(Ident<'a>, SymbolTable<'a>)>,
}

impl<'a> SymbolTableZipper<'a> {
    pub fn enter_scope(&mut self, ident: Ident<'a>) -> Result<(), SymbolsError> {
        let mut temp_cursor = self
            .cursor
            .remove_scope(&ident)
            .ok_or(SymbolsError::ScopeNotFound(ident.clone()))?;

        std::mem::swap(&mut self.cursor, &mut temp_cursor);
        self.breadcrumb.push((ident, temp_cursor));

        Ok(())
    }

    pub fn exit_scope(&mut self) -> Result<(), SymbolsError> {
        let (ident, mut temp_scope) = self
            .breadcrumb
            .pop()
            .ok_or(SymbolsError::MovePastGlobelScope)?;

        std::mem::swap(&mut self.cursor, &mut temp_scope);
        self.cursor.insert_scope(ident, temp_scope)?;

        Ok(())
    }

    pub fn locate(&self, ident: &Ident<'a>) -> Option<&VarDecl<'a>> {
        if let Some(value) = self.cursor.locate(ident) {
            return Some(value);
        }

        for scope in self.breadcrumb.iter().rev() {
            if let Some(value) = scope.1.locate(ident) {
                return Some(value);
            }
        }

        return None;
    }

    pub fn locate_current_scope(&self, ident: &Ident<'a>) -> Option<&VarDecl<'a>> {
        self.cursor.locate(ident)
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
