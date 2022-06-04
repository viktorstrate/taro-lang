use std::collections::HashMap;

use crate::ast::node::{
    identifier::{Ident, Identifiable},
    statement::VarDecl,
};

#[derive(Debug)]
pub enum SymbolsError<'a> {
    SymbolAlreadyExistsInScope(Ident<'a>),
    ScopeNotFound(Ident<'a>),
    MovePastGlobelScope,
}

#[derive(Default, Debug)]
pub struct SymbolTable<'a> {
    table: HashMap<Ident<'a>, VarDecl<'a>>,
    scopes: HashMap<Ident<'a>, SymbolTable<'a>>,
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
