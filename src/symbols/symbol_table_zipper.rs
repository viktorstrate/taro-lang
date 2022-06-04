use crate::ast::node::identifier::Ident;

use super::symbol_table::{SymbolTable, SymbolValue, SymbolsError};

#[derive(Debug)]
pub struct SymbolTableZipper<'a> {
    cursor: SymbolTable<'a>,
    breadcrumb: Vec<(Ident<'a>, SymbolTable<'a>)>,
}

impl<'a> Into<SymbolTableZipper<'a>> for SymbolTable<'a> {
    fn into(self) -> SymbolTableZipper<'a> {
        SymbolTableZipper {
            cursor: self,
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

    pub fn locate(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
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

    pub fn locate_current_scope(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
        self.cursor.locate(ident)
    }
}
