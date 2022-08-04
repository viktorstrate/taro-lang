use crate::ir::{
    ast_walker::{AstWalker, ScopeValue},
    context::IrCtx,
    node::{
        identifier::{Ident, ResolvedIdentValue},
        statement::Stmt::{self, VariableDecl},
        type_signature::BUILTIN_TYPES,
    },
};

use super::symbol_table::{SymbolTable, SymbolValue, SymbolsError};

pub struct SymbolCollector<'a, 'ctx> {
    ctx: IrCtx<'a, 'ctx>,
}

impl<'a: 'ctx, 'ctx> AstWalker<'a, 'ctx> for SymbolCollector<'a, 'ctx> {
    type Scope = SymbolTable<'a, 'ctx>;
    type Error = SymbolsError<'a, 'ctx>;

    fn visit_begin(&'ctx mut self, scope: &mut Self::Scope) -> Result<(), Self::Error> {
        for builtin_type in BUILTIN_TYPES {
            scope.insert(SymbolValue::BuiltinType(
                self.ctx.make_builtin_ident(*builtin_type),
            ))?;
        }

        Ok(())
    }

    fn visit_scope_begin(
        &'ctx mut self,
        parent: &mut SymbolTable<'a, 'ctx>,
        value: ScopeValue<'a, 'ctx>,
    ) -> Result<SymbolTable<'a, 'ctx>, Self::Error> {
        let mut new_scope = SymbolTable::default();

        match value {
            ScopeValue::Func(func) => {
                parent.insert(SymbolValue::FuncDecl(func))?;
                for arg in &func.args {
                    new_scope.insert(SymbolValue::FuncArg(arg))?;
                }
            }
            ScopeValue::Struct(st) => {
                parent.insert(SymbolValue::StructDecl(st))?;
                for attr in st.attrs {
                    new_scope.insert(SymbolValue::StructAttr(attr))?;
                }
            }
            ScopeValue::StructInit(st_init) => {
                new_scope.insert(SymbolValue::StructInit(st_init))?;
            }
            ScopeValue::Enum(enm) => {
                parent.insert(SymbolValue::EnumDecl(enm))?;
            }
        }

        Ok(new_scope)
    }

    fn visit_scope_end(
        &mut self,
        parent: &'ctx mut SymbolTable<'a, 'ctx>,
        child: SymbolTable<'a, 'ctx>,
        value: ScopeValue<'a, 'ctx>,
    ) -> Result<(), Self::Error> {
        // save child scope in parent scope
        match value {
            ScopeValue::Func(func) => parent.insert_scope(func.name.clone(), child).map(|_| ()),
            ScopeValue::Struct(st) => parent.insert_scope(st.name.clone(), child).map(|_| ()),
            ScopeValue::StructInit(st_init) => parent
                .insert_scope(st_init.scope_name.clone(), child)
                .map(|_| ()),
            ScopeValue::Enum(enm) => parent.insert_scope(enm.name.clone(), child).map(|_| ()),
        }
    }

    fn visit_stmt(
        &'ctx mut self,
        scope: &mut SymbolTable<'a, 'ctx>,
        stmt: &'ctx mut Stmt<'a, 'ctx>,
    ) -> Result<(), Self::Error> {
        match stmt {
            VariableDecl(var_decl) => scope.insert(SymbolValue::VarDecl(var_decl)).map(|_| ()),
            // FunctionDecl(func) => scope.insert(func.clone().into()).map(|_| ()),
            _ => Ok(()),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use crate::{
//         ir::ast_walker::walk_ast,
//         parser::parse_ast,
//         symbols::{symbol_table::SymbolsError, symbol_walker::SymbolCollector},
//     };

//     #[test]
//     fn test_symbol_shadowing() {
//         let mut ast = parse_ast("let x = 2; let x = true").unwrap();
//         let mut collector = SymbolCollector::default();
//         let result = walk_ast(&mut collector, &mut ast);
//         assert_matches!(result, Ok(_));
//     }

//     #[test]
//     fn test_existing_symbol_error() {
//         let mut ast = parse_ast("func f() {}; func f() {}").unwrap();
//         let mut collector = SymbolCollector::default();
//         let result = walk_ast(&mut collector, &mut ast);
//         assert_matches!(result, Err(SymbolsError::SymbolAlreadyExistsInScope(_)))
//     }
// }
