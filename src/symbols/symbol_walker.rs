use crate::ast::{
    ast_walker::AstWalker,
    node::{
        function::Function,
        statement::Stmt::{self, FunctionDecl, VariableDecl},
        structure::Struct,
    },
};

use super::symbol_table::{SymbolTable, SymbolValue, SymbolsError};

#[derive(Default)]
pub struct SymbolCollector {}

impl<'a> AstWalker<'a> for SymbolCollector {
    type Scope = SymbolTable<'a>;
    type Error = SymbolsError<'a>;

    fn visit_scope_begin(
        &mut self,
        _parent: &mut SymbolTable<'a>,
        func: &mut Function<'a>,
    ) -> Result<SymbolTable<'a>, Self::Error> {
        let mut new_scope = SymbolTable::default();

        for arg in &func.args {
            new_scope.insert(SymbolValue::FuncArg(arg.clone()))?;
        }

        Ok(new_scope)
    }

    fn visit_scope_end(
        &mut self,
        parent: &mut SymbolTable<'a>,
        child: SymbolTable<'a>,
        func: &mut Function<'a>,
    ) -> Result<(), Self::Error> {
        // save child scope in parent scope
        parent.insert_scope(func.name.clone(), child).map(|_| ())
    }

    fn visit_stmt(
        &mut self,
        scope: &mut SymbolTable<'a>,
        stmt: &mut Stmt<'a>,
    ) -> Result<(), Self::Error> {
        match stmt {
            VariableDecl(var_decl) => scope.insert(var_decl.clone().into()).map(|_| ()),
            FunctionDecl(func_decl) => scope.insert(func_decl.clone().into()).map(|_| ()),
            _ => Ok(()),
        }
    }

    fn visit_struct_decl(
        &mut self,
        scope: &mut SymbolTable<'a>,
        st: &mut Struct<'a>,
    ) -> Result<(), Self::Error> {
        scope.insert(SymbolValue::StructDecl(st.clone()))?;

        let mut struct_scope = SymbolTable::default();

        for attr in &st.attrs {
            struct_scope.insert(SymbolValue::StructAttr(attr.clone()))?;
        }

        scope.insert_scope(st.name.clone(), struct_scope)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::{
            ast_walker::walk_ast,
            node::{
                expression::Expr,
                identifier::{Ident, Identifiable},
            },
        },
        parser::parse_ast,
        symbols::{symbol_table::SymbolValue, symbol_walker::SymbolCollector},
    };

    #[test]
    fn test_existing_symbol_error() {
        let mut ast = parse_ast("let x = 2; let x = 3").unwrap();
        let mut collector = SymbolCollector::default();
        let result = walk_ast(&mut collector, &mut ast);
        assert_matches!(result, Err(_));
    }

    #[test]
    fn test_locate_symbol() {
        let mut ast = parse_ast("let x: Boolean = true").unwrap();
        let mut collector = SymbolCollector::default();
        let symtable = walk_ast(&mut collector, &mut ast).unwrap();
        let sym_val = symtable.locate(&Ident::new_unplaced("x")).unwrap();

        assert_eq!(*sym_val.name(), Ident::new_unplaced("x"));
        match sym_val {
            SymbolValue::VarDecl(var_decl) => {
                assert_matches!(var_decl.value, Expr::BoolLiteral(true));
            }
            _ => assert!(false),
        }
    }
}
