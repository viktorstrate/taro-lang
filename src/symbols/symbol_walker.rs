use crate::ast::{ast_walker::AstWalker, Ident};

use super::{SymbolTable, SymbolsError};

#[derive(Default)]
pub struct SymbolCollector {}

impl<'a> AstWalker<'a> for SymbolCollector {
    type Scope = SymbolTable<'a>;
    type Error = SymbolsError;

    fn visit_scope_end(
        &mut self,
        parent: &mut Self::Scope,
        child: Self::Scope,
        scope_ident: &Ident<'a>,
    ) -> Result<(), SymbolsError> {
        // save child scope in parent scope
        parent.insert_scope(scope_ident.clone(), child).map(|_| ())
    }

    fn visit_declaration(
        &mut self,
        scope: &mut Self::Scope,
        decl: &crate::ast::VarDecl<'a>,
    ) -> Result<(), SymbolsError> {
        scope.insert(decl.clone()).map(|_| ())
    }
}
