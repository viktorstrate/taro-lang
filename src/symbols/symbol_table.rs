use std::collections::HashMap;

use crate::ast::node::{
    function::FuncDecl,
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
    table: HashMap<Ident<'a>, SymbolValue<'a>>,
    scopes: HashMap<Ident<'a>, SymbolTable<'a>>,
}

#[derive(Debug)]
pub enum SymbolValue<'a> {
    VarDecl(VarDecl<'a>),
    FuncDecl(FuncDecl<'a>),
}

impl<'a> From<FuncDecl<'a>> for SymbolValue<'a> {
    fn from(func: FuncDecl<'a>) -> Self {
        Self::FuncDecl(func)
    }
}

impl<'a> From<VarDecl<'a>> for SymbolValue<'a> {
    fn from(var: VarDecl<'a>) -> Self {
        Self::VarDecl(var)
    }
}

impl<'a> Identifiable<'a> for SymbolValue<'a> {
    fn name(&self) -> &Ident<'a> {
        match self {
            SymbolValue::VarDecl(var) => var.name(),
            SymbolValue::FuncDecl(func) => func.name(),
        }
    }
}

impl<'a> SymbolTable<'a> {
    pub fn insert(
        &mut self,
        val: SymbolValue<'a>,
    ) -> Result<&mut SymbolValue<'a>, SymbolsError<'a>> {
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

    pub fn locate(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
        self.table.get(ident)
    }
}
