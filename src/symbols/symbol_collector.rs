use crate::ir::{
    context::IrCtx,
    ir_walker::{IrWalker, ScopeValue},
    node::{statement::Stmt, type_signature::BUILTIN_TYPES, NodeRef},
};

use super::symbol_table::{SymbolTable, SymbolValueItem, SymbolsError};

pub struct SymbolCollector {}

impl<'a> IrWalker<'a> for SymbolCollector {
    type Scope = SymbolTable<'a>;
    type Error = SymbolsError<'a>;

    fn visit_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        for builtin_type in BUILTIN_TYPES {
            let sym_val = SymbolValueItem::BuiltinType(ctx.make_builtin_ident(*builtin_type));
            scope.insert(ctx, sym_val)?;
        }

        Ok(())
    }

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        parent: &mut SymbolTable<'a>,
        value: ScopeValue<'a>,
    ) -> Result<SymbolTable<'a>, Self::Error> {
        let mut new_scope = SymbolTable::default();

        match value {
            ScopeValue::Func(func) => {
                parent.insert(ctx, SymbolValueItem::FuncDecl(func))?;
                for arg in ctx[func].args.clone() {
                    new_scope.insert(ctx, SymbolValueItem::FuncArg(arg))?;
                }
            }
            ScopeValue::Struct(st) => {
                parent.insert(ctx, SymbolValueItem::StructDecl(st))?;
                for attr in ctx[st].attrs.clone() {
                    new_scope.insert(ctx, SymbolValueItem::StructAttr(attr))?;
                }
            }
            ScopeValue::StructInit(st_init) => {
                new_scope.insert(ctx, SymbolValueItem::StructInit(st_init))?;
            }
            ScopeValue::Enum(enm) => {
                parent.insert(ctx, SymbolValueItem::EnumDecl(enm))?;
            }
        }

        Ok(new_scope)
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        parent: &mut SymbolTable<'a>,
        child: SymbolTable<'a>,
        value: ScopeValue<'a>,
    ) -> Result<(), Self::Error> {
        // save child scope in parent scope
        match value {
            ScopeValue::Func(func) => parent.insert_scope(ctx, ctx[func].name, child).map(|_| ()),
            ScopeValue::Struct(st) => parent.insert_scope(ctx, ctx[st].name, child).map(|_| ()),
            ScopeValue::StructInit(st_init) => parent
                .insert_scope(ctx, ctx[st_init].scope_name, child)
                .map(|_| ()),
            ScopeValue::Enum(enm) => parent.insert_scope(ctx, ctx[enm].name, child).map(|_| ()),
        }
    }

    fn visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut SymbolTable<'a>,
        stmt: NodeRef<'a, Stmt<'a>>,
    ) -> Result<(), Self::Error> {
        match &ctx[stmt] {
            Stmt::VariableDecl(var_decl) => {
                let decl = *var_decl;
                scope
                    .insert(ctx, SymbolValueItem::VarDecl(decl))
                    .map(|_| ())
            }
            Stmt::FunctionDecl(func) => {
                let fnc = *func;
                scope.insert(ctx, (fnc).into()).map(|_| ())
            }
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
