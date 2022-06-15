use crate::ast::node::identifier::{Ident, Identifiable};

use super::symbol_table::{SymbolTable, SymbolValue, SymbolsError};

#[derive(Debug)]
struct SymbolTableZipperBreadcrumb<'a> {
    scope_name: Ident<'a>,
    sym_table: SymbolTable<'a>,
    visited_symbols: Vec<SymbolValue<'a>>,
}

#[derive(Debug)]
pub struct SymbolTableZipper<'a> {
    cursor: SymbolTable<'a>,
    visited_symbols: Vec<SymbolValue<'a>>,
    breadcrumb: Vec<SymbolTableZipperBreadcrumb<'a>>,
}

impl<'a> Into<SymbolTableZipper<'a>> for SymbolTable<'a> {
    fn into(self) -> SymbolTableZipper<'a> {
        SymbolTableZipper {
            cursor: self,
            visited_symbols: Vec::new(),
            breadcrumb: Vec::new(),
        }
    }
}

impl<'a> SymbolTableZipper<'a> {
    pub fn enter_scope(&mut self, ident: Ident<'a>) -> Result<(), SymbolsError> {
        let mut temp_cursor = self
            .cursor
            .remove_scope(&ident)
            .ok_or(SymbolsError::ScopeNotFound(ident.clone()))?;

        std::mem::swap(&mut self.cursor, &mut temp_cursor);
        self.breadcrumb.push(SymbolTableZipperBreadcrumb {
            scope_name: ident,
            sym_table: temp_cursor,
            visited_symbols: self.visited_symbols.drain(..).collect(),
        });

        Ok(())
    }

    pub fn exit_scope(&mut self) -> Result<(), SymbolsError> {
        let mut breadcrumb = self
            .breadcrumb
            .pop()
            .ok_or(SymbolsError::MovePastGlobalScope)?;

        // move the visited symbols back
        for visited_sym in self.visited_symbols.drain(..).rev() {
            self.cursor.ordered_symbols.push_front(visited_sym);
        }

        std::mem::swap(&mut self.cursor, &mut breadcrumb.sym_table);
        self.cursor
            .insert_scope(breadcrumb.scope_name, breadcrumb.sym_table)?;

        self.visited_symbols = breadcrumb.visited_symbols;

        Ok(())
    }

    pub fn lookup(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
        if let Some(value) = self.lookup_current_scope(ident) {
            return Some(value);
        }

        for scope in self.breadcrumb.iter().rev() {
            if let Some(value) = scope.sym_table.lookup_global_table(ident) {
                return Some(value);
            }

            if let Some(value) = scope
                .visited_symbols
                .iter()
                .rev()
                .find(|sym| *sym.name() == *ident)
            {
                return Some(value);
            }
        }

        return None;
    }

    pub fn lookup_current_scope(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
        if let Some(sym) = self.cursor.lookup_global_table(ident) {
            return Some(sym);
        }

        self.visited_symbols
            .iter()
            .rev()
            .find(|sym| *sym.name() == *ident)
    }

    pub fn visit_next_symbol(&mut self) {
        let sym = self
            .cursor
            .pop_ordered_symbol()
            .expect("visit_next_symbol() should match up");

        self.visited_symbols.push(sym);
    }

    pub fn reset(&mut self) {
        while !self.breadcrumb.is_empty() {
            self.exit_scope().expect("while guard ensures");
        }

        while !self.visited_symbols.is_empty() {
            for sym in self.visited_symbols.drain(..) {
                self.cursor.insert(sym).expect("reinserting symbol");
            }
        }
    }
}
