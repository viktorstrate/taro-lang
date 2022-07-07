use crate::ast::node::identifier::{Ident, Identifiable};

use super::{SymbolTable, SymbolValue, SymbolsError};

#[derive(Debug)]
struct SymbolTableZipperBreadcrumb<'a> {
    scope_name: Ident<'a>,
    sym_table: SymbolTable<'a>,
    visited_symbols: usize,
}

/// Structure used to keep track of the current position in a symbol table.
#[derive(Debug)]
pub struct SymbolTableZipper<'a> {
    cursor: SymbolTable<'a>,
    visited_symbols: usize,
    breadcrumb: Vec<SymbolTableZipperBreadcrumb<'a>>,
}

impl<'a> Into<SymbolTableZipper<'a>> for SymbolTable<'a> {
    fn into(self) -> SymbolTableZipper<'a> {
        SymbolTableZipper {
            cursor: self,
            visited_symbols: 0,
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
            visited_symbols: self.visited_symbols,
        });

        Ok(())
    }

    pub fn exit_scope(&mut self) -> Result<(), SymbolsError> {
        let mut breadcrumb = self
            .breadcrumb
            .pop()
            .ok_or(SymbolsError::MovePastGlobalScope)?;

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

            if let Some(value) = SymbolTableZipper::locate_visited_symbol(
                &scope.sym_table,
                scope.visited_symbols,
                ident,
            ) {
                return Some(value);
            }
        }

        return None;
    }

    fn locate_visited_symbol<'b>(
        sym_table: &'b SymbolTable<'a>,
        visited_symbols: usize,
        ident: &Ident<'a>,
    ) -> Option<&'b SymbolValue<'a>> {
        sym_table
            .ordered_symbols
            .iter()
            .take(visited_symbols)
            .rev()
            .find(|sym| *sym.name() == *ident)
    }

    pub fn lookup_current_scope(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
        if let Some(sym) = self.cursor.lookup_global_table(ident) {
            return Some(sym);
        }

        SymbolTableZipper::locate_visited_symbol(&self.cursor, self.visited_symbols, ident)
    }

    pub fn visit_next_symbol(&mut self) {
        debug_assert!(self.visited_symbols <= self.cursor.ordered_symbols.len());
        self.visited_symbols += 1;
    }

    pub fn reset(&mut self) {
        while !self.breadcrumb.is_empty() {
            self.exit_scope().expect("while guard ensures");
        }

        self.visited_symbols = 0;
    }
}
