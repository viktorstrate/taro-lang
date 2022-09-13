use crate::ir::{
    context::IrCtx,
    node::identifier::{Ident, IdentKey, Identifiable},
};

use super::{SymbolCollectionError, SymbolTable, SymbolValue};

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
    pub fn enter_scope(
        &mut self,
        ctx: &IrCtx<'a>,
        ident: Ident<'a>,
    ) -> Result<(), SymbolCollectionError<'a>> {
        let mut temp_cursor = self
            .cursor
            .remove_scope(ctx, ident)
            .ok_or(SymbolCollectionError::ScopeNotFound(ident))?;

        std::mem::swap(&mut self.cursor, &mut temp_cursor);
        self.breadcrumb.push(SymbolTableZipperBreadcrumb {
            scope_name: ident,
            sym_table: temp_cursor,
            visited_symbols: self.visited_symbols,
        });

        self.visited_symbols = 0;

        Ok(())
    }

    pub fn exit_scope(&mut self, ctx: &IrCtx<'a>) -> Result<(), SymbolCollectionError<'a>> {
        let mut breadcrumb = self.breadcrumb.pop().expect("move past global scope");
        // .ok_or(SymbolCollectionError::MovePastGlobalScope)?;

        std::mem::swap(&mut self.cursor, &mut breadcrumb.sym_table);
        self.cursor
            .insert_scope(ctx, breadcrumb.scope_name, breadcrumb.sym_table)?;

        self.visited_symbols = breadcrumb.visited_symbols;

        Ok(())
    }

    pub fn lookup(&self, ctx: &IrCtx<'a>, ident: Ident<'a>) -> Option<SymbolValue<'a>> {
        if let Some(value) = self.lookup_current_scope(ctx, ident) {
            return Some(*value);
        }

        for scope in self.breadcrumb.iter().rev() {
            if let Some(value) = scope.sym_table.lookup_global_table(ctx, ident) {
                return Some(*value);
            }

            if let Some(value) = SymbolTableZipper::locate_visited_symbol(
                ctx,
                &scope.sym_table,
                scope.visited_symbols,
                ident,
            ) {
                return Some(*value);
            }
        }

        return None;
    }

    fn locate_visited_symbol<'b>(
        ctx: &IrCtx<'a>,
        sym_table: &'b SymbolTable<'a>,
        visited_symbols: usize,
        ident: Ident<'a>,
    ) -> Option<&'b SymbolValue<'a>> {
        sym_table
            .ordered_symbols
            .iter()
            .take(visited_symbols)
            .rev()
            .find(|sym| IdentKey::idents_eq(ctx, ctx[**sym].name(ctx), ident))
    }

    pub fn lookup_current_scope(
        &self,
        ctx: &IrCtx<'a>,
        ident: Ident<'a>,
    ) -> Option<&SymbolValue<'a>> {
        if let Some(sym) = self.cursor.lookup_global_table(ctx, ident) {
            return Some(sym);
        }

        SymbolTableZipper::locate_visited_symbol(ctx, &self.cursor, self.visited_symbols, ident)
    }

    pub fn visit_next_symbol(&mut self, _ctx: &IrCtx<'a>) {
        debug_assert!(self.visited_symbols < self.cursor.ordered_symbols.len());
        self.visited_symbols += 1;
    }

    pub fn reset(&mut self, ctx: &IrCtx<'a>) {
        while !self.breadcrumb.is_empty() {
            self.exit_scope(ctx).expect("while guard ensures");
        }

        self.visited_symbols = 0;
    }
}
