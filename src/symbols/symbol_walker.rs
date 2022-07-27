use crate::ast::{
    ast_walker::{AstWalker, ScopeValue},
    node::{
        statement::Stmt::{self, FunctionDecl, VariableDecl},
        structure::Struct,
    },
};

use super::{
    builtin_types::BUILTIN_TYPES,
    symbol_table::{SymbolTable, SymbolValue, SymbolsError},
};

#[derive(Default)]
pub struct SymbolCollector {}

impl<'a> AstWalker<'a> for SymbolCollector {
    type Scope = SymbolTable<'a>;
    type Error = SymbolsError<'a>;

    fn visit_begin(&mut self, scope: &mut Self::Scope) -> Result<(), Self::Error> {
        for builtin_type in BUILTIN_TYPES {
            scope.insert(SymbolValue::BuiltinType(builtin_type.clone()))?;
        }

        Ok(())
    }

    fn visit_scope_begin(
        &mut self,
        _parent: &mut SymbolTable<'a>,
        value: ScopeValue<'a, '_>,
    ) -> Result<SymbolTable<'a>, Self::Error> {
        let mut new_scope = SymbolTable::default();

        match value {
            ScopeValue::Func(func) => {
                for arg in &func.args {
                    new_scope.insert(SymbolValue::FuncArg(arg.clone()))?;
                }
            }
            ScopeValue::Struct(st) => {
                for attr in &st.attrs {
                    new_scope.insert(SymbolValue::StructAttr(attr.clone()))?;
                }
            } // ScopeValue::StructInit(_) => {}
        }

        Ok(new_scope)
    }

    fn visit_scope_end(
        &mut self,
        parent: &mut SymbolTable<'a>,
        child: SymbolTable<'a>,
        value: ScopeValue<'a, '_>,
    ) -> Result<(), Self::Error> {
        // save child scope in parent scope
        match value {
            ScopeValue::Func(func) => parent.insert_scope(func.name.clone(), child).map(|_| ()),
            ScopeValue::Struct(st) => parent.insert_scope(st.name.clone(), child).map(|_| ()),
            // ScopeValue::StructInit(st_init) => {
            //     parent.insert_scope(st_init.name.clone(), child).map(|_| ())
            // }
        }
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::ast_walker::walk_ast,
        parser::parse_ast,
        symbols::{symbol_table::SymbolsError, symbol_walker::SymbolCollector},
    };

    #[test]
    fn test_symbol_shadowing() {
        let mut ast = parse_ast("let x = 2; let x = true").unwrap();
        let mut collector = SymbolCollector::default();
        let result = walk_ast(&mut collector, &mut ast);
        assert_matches!(result, Ok(_));
    }

    #[test]
    fn test_existing_symbol_error() {
        let mut ast = parse_ast("func f() {}; func f() {}").unwrap();
        let mut collector = SymbolCollector::default();
        let result = walk_ast(&mut collector, &mut ast);
        assert_matches!(result, Err(SymbolsError::SymbolAlreadyExistsInScope(_)))
    }
}
