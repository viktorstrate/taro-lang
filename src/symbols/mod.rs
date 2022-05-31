use std::collections::HashMap;

use crate::ast::{self, Ident, Identifiable, VarDecl};

pub mod symbol_walker;

#[derive(Default, Debug)]
pub struct SymbolTable<'a> {
    table: HashMap<Ident<'a>, VarDecl<'a>>,
    scopes: HashMap<Ident<'a>, SymbolTable<'a>>,
}

// #[derive(PartialEq, Debug, Clone)]
// pub enum SymbolVal<'a> {
//     VarDecl(ast::VarDecl<'a>),
// }

// impl<'a> Identifiable for SymbolVal<'a> {
//     fn name(&self) -> &Ident {
//         match self {
//             Self::VarDecl(vardecl) => vardecl.name(),
//         }
//     }
// }

pub enum SymbolsError {
    SymbolAlreadyExistsInScope,
}

impl<'a> SymbolTable<'a> {
    pub fn insert(&mut self, val: VarDecl<'a>) {
        let key: Ident<'a> = val.name().clone();
        self.table.insert(key, val);
    }

    pub fn insert_scope(&mut self, ident: Ident<'a>) -> Result<&mut SymbolTable<'a>, SymbolsError> {
        self.scopes
            .try_insert(ident, SymbolTable::default())
            .map_err(|_| SymbolsError::SymbolAlreadyExistsInScope)
    }
}
