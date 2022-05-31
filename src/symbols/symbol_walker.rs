use crate::ast::{ast_walker::AstWalker, Stmt};

use super::SymbolTable;

pub struct SymbolCollector<'a> {
    scope: SymbolTable<'a>,
}

impl<'a> Default for SymbolCollector<'a> {
    fn default() -> Self {
        Self {
            scope: SymbolTable::default(),
        }
    }
}

impl<'a> AstWalker<'a> for SymbolCollector<'a> {
    fn visit_stmt(&mut self, stmt: &Stmt<'a>) {
        match stmt {
            Stmt::VarDecl(var_decl) => self.scope.insert(var_decl.clone()),
            _ => {}
        }
    }
}
