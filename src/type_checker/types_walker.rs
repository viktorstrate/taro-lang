use crate::{ast::ast_walker::AstWalker, symbols::SymbolTable};

pub struct TypeChecker<'a> {
    symbols: SymbolTable<'a>,
}

impl<'a> AstWalker<'a> for TypeChecker<'a> {}
