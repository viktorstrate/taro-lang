use std::collections::{HashMap, VecDeque};

use id_arena::Id;

use crate::ir::{
    context::{IrArenaType, IrCtx},
    node::{
        enumeration::Enum,
        function::{Function, FunctionArg},
        identifier::{Ident, IdentKey, Identifiable, ResolvedIdentValue},
        statement::VarDecl,
        structure::{Struct, StructAttr, StructInit},
        type_signature::{TypeEvalError, TypeSignature, Typed},
        NodeRef,
    },
};

use self::symbol_table_zipper::SymbolTableZipper;

pub mod symbol_table_zipper;

#[derive(Debug)]
pub enum SymbolsError<'a> {
    SymbolAlreadyExistsInScope(Ident<'a>),
    ScopeNotFound(Ident<'a>),
    MovePastGlobalScope,
}

#[derive(Default, Debug)]
pub struct SymbolTable<'a> {
    /// Symbols that are available from the entire scope
    pub(super) scope_global_table: HashMap<IdentKey<'a>, SymbolValue<'a>>,
    /// An list for order dependent symbols such as variable declarations,
    /// where symbols are first available after they have been declared.
    pub(super) ordered_symbols: VecDeque<SymbolValue<'a>>,
    /// Nested scopes such as function bodies and struct definitions.
    pub(super) scopes: HashMap<IdentKey<'a>, SymbolTable<'a>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct SymbolValue<'a> {
    id: Id<SymbolValueItem<'a>>,
}

/// A value returned from a symbol lookup
#[derive(Debug, Clone, Copy)]
pub enum SymbolValueItem<'a> {
    BuiltinType(Ident<'a>),
    VarDecl(NodeRef<'a, VarDecl<'a>>),
    FuncDecl(NodeRef<'a, Function<'a>>),
    FuncArg(NodeRef<'a, FunctionArg<'a>>),
    StructDecl(NodeRef<'a, Struct<'a>>),
    StructAttr(NodeRef<'a, StructAttr<'a>>),
    StructInit(NodeRef<'a, StructInit<'a>>),
    EnumDecl(NodeRef<'a, Enum<'a>>),
}

impl<'a> Into<Id<SymbolValueItem<'a>>> for SymbolValue<'a> {
    fn into(self) -> Id<SymbolValueItem<'a>> {
        self.id
    }
}

impl<'a> From<Id<SymbolValueItem<'a>>> for SymbolValue<'a> {
    fn from(id: Id<SymbolValueItem<'a>>) -> Self {
        Self { id }
    }
}

impl<'a> From<NodeRef<'a, Function<'a>>> for SymbolValueItem<'a> {
    fn from(func: NodeRef<'a, Function<'a>>) -> Self {
        Self::FuncDecl(func)
    }
}

impl<'a> From<NodeRef<'a, VarDecl<'a>>> for SymbolValueItem<'a> {
    fn from(var: NodeRef<'a, VarDecl<'a>>) -> Self {
        Self::VarDecl(var)
    }
}

impl<'a> Identifiable<'a> for SymbolValueItem<'a> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a> {
        match self {
            SymbolValueItem::BuiltinType(builtin) => *builtin,
            SymbolValueItem::VarDecl(var) => ctx[*var].name(ctx),
            SymbolValueItem::FuncDecl(func) => ctx[*func].name(ctx),
            SymbolValueItem::FuncArg(arg) => ctx[*arg].name(ctx),
            SymbolValueItem::StructDecl(st) => ctx[*st].name(ctx),
            SymbolValueItem::StructAttr(attr) => ctx[*attr].name(ctx),
            SymbolValueItem::StructInit(st_init) => ctx[*st_init].name(ctx),
            SymbolValueItem::EnumDecl(enm) => ctx[*enm].name(ctx),
        }
    }
}

impl<'a> IrArenaType<'a> for SymbolValueItem<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b id_arena::Arena<Self> {
        &ctx.symbols
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut id_arena::Arena<Self> {
        &mut ctx.symbols
    }
}

impl SymbolValueItem<'_> {
    fn is_order_dependent(&self) -> bool {
        match self {
            SymbolValueItem::VarDecl(_) => true,
            _ => false,
        }
    }
}

impl<'a> Typed<'a> for SymbolValue<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match ctx[*self].clone() {
            SymbolValueItem::BuiltinType(builtin) => match &ctx[builtin] {
                crate::ir::node::identifier::IdentValue::Resolved(
                    ResolvedIdentValue::BuiltinType(builtin_type),
                ) => Ok(ctx.get_builtin_type_sig(*builtin_type)),
                _ => unreachable!("ident should resolve to builtin type"),
            },
            SymbolValueItem::VarDecl(var) => var.eval_type(symbols, ctx),
            SymbolValueItem::FuncDecl(decl) => decl.eval_type(symbols, ctx),
            SymbolValueItem::FuncArg(arg) => arg.eval_type(symbols, ctx),
            SymbolValueItem::StructDecl(st) => st.eval_type(symbols, ctx),
            SymbolValueItem::StructAttr(attr) => attr.eval_type(symbols, ctx),
            SymbolValueItem::StructInit(st_init) => st_init.eval_type(symbols, ctx),
            SymbolValueItem::EnumDecl(enm) => enm.eval_type(symbols, ctx),
        }
    }

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        match ctx[*self].clone() {
            SymbolValueItem::BuiltinType(_) => None,
            SymbolValueItem::VarDecl(var) => var.specified_type(ctx),
            SymbolValueItem::FuncDecl(decl) => decl.specified_type(ctx),
            SymbolValueItem::FuncArg(arg) => arg.specified_type(ctx),
            SymbolValueItem::StructDecl(st) => st.specified_type(ctx),
            SymbolValueItem::StructAttr(attr) => attr.specified_type(ctx),
            SymbolValueItem::StructInit(st_init) => st_init.specified_type(ctx),
            SymbolValueItem::EnumDecl(enm) => enm.specified_type(ctx),
        }
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        match ctx[*self].clone() {
            SymbolValueItem::BuiltinType(_) => Ok(()),
            SymbolValueItem::VarDecl(var) => var.specify_type(ctx, new_type),
            SymbolValueItem::FuncDecl(decl) => decl.specify_type(ctx, new_type),
            SymbolValueItem::FuncArg(arg) => arg.specify_type(ctx, new_type),
            SymbolValueItem::StructDecl(st) => st.specify_type(ctx, new_type),
            SymbolValueItem::StructAttr(attr) => attr.specify_type(ctx, new_type),
            SymbolValueItem::StructInit(st_init) => st_init.specify_type(ctx, new_type),
            SymbolValueItem::EnumDecl(enm) => enm.specify_type(ctx, new_type),
        }
    }
}

impl<'a> SymbolTable<'a> {
    pub fn insert(
        &mut self,
        ctx: &mut IrCtx<'a>,
        val: SymbolValueItem<'a>,
    ) -> Result<SymbolValue<'a>, SymbolsError<'a>> {
        let new_sym = ctx.make_symbol(val);
        let val = &ctx[new_sym];
        if val.is_order_dependent() {
            self.ordered_symbols.push_back(new_sym);
        } else {
            let ident = val.name(ctx);
            let key = IdentKey::from_ident(ctx, ident);
            self.scope_global_table
                .try_insert(key, new_sym)
                .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(ident))?;
        }
        Ok(new_sym)
    }

    pub fn insert_scope(
        &mut self,
        ctx: &IrCtx<'a>,
        ident: Ident<'a>,
        scope: SymbolTable<'a>,
    ) -> Result<&mut SymbolTable<'a>, SymbolsError<'a>> {
        self.scopes
            .try_insert(IdentKey::from_ident(ctx, ident), scope)
            .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(ident))
    }

    pub fn remove_scope(&mut self, ctx: &IrCtx<'a>, ident: Ident<'a>) -> Option<SymbolTable<'a>> {
        self.scopes.remove(&IdentKey::from_ident(ctx, ident))
    }

    pub fn lookup_global_table(
        &self,
        ctx: &IrCtx<'a>,
        ident: Ident<'a>,
    ) -> Option<&SymbolValue<'a>> {
        self.scope_global_table
            .get(&IdentKey::from_ident(ctx, ident))
    }
}

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use crate::{
//         ir::{ast_walker::walk_ast, node::expression::Expr},
//         parser::parse_ast,
//         symbols::symbol_walker::SymbolCollector,
//     };

//     use super::*;

//     #[test]
//     fn test_locate_ordered_symbol() {
//         let mut ast = parse_ast("let x: Boolean = true").unwrap();
//         let mut collector = SymbolCollector::default();
//         let symtable = walk_ast(&mut collector, &mut ast).unwrap();
//         let sym_val = symtable.ordered_symbols.front().unwrap();

//         assert_eq!(*sym_val.name(), Ident::new_unplaced("x"));
//         match sym_val {
//             SymbolValue::VarDecl(var_decl) => {
//                 assert_matches!(var_decl.value, Expr::BoolLiteral(true));
//             }
//             _ => assert!(false),
//         }
//     }
// }
