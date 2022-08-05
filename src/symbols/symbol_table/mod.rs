use std::collections::{HashMap, VecDeque};

use id_arena::{Id};

use crate::ir::{
    context::IrCtx,
    node::{
        enumeration::Enum,
        function::{Function, FunctionArg},
        identifier::{Ident, Identifiable},
        statement::VarDecl,
        structure::{Struct, StructAttr, StructInit},
    },
};

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
    pub(super) scope_global_table: HashMap<Ident<'a>, SymbolValue<'a>>,
    /// An list for order dependent symbols such as variable declarations,
    /// where symbols are first available after they have been declared.
    pub(super) ordered_symbols: VecDeque<SymbolValue<'a>>,
    /// Nested scopes such as function bodies and struct definitions.
    pub(super) scopes: HashMap<Ident<'a>, SymbolTable<'a>>,
}

pub type SymbolValue<'a> = Id<SymbolValueItem<'a>>;

/// A value returned from a symbol lookup
#[derive(Debug)]
pub enum SymbolValueItem<'a> {
    BuiltinType(Ident<'a>),
    VarDecl(Id<VarDecl<'a>>),
    FuncDecl(Id<Function<'a>>),
    FuncArg(Id<FunctionArg<'a>>),
    StructDecl(Id<Struct<'a>>),
    StructAttr(Id<StructAttr<'a>>),
    StructInit(Id<StructInit<'a>>),
    EnumDecl(Id<Enum<'a>>),
}

impl<'a> From<Id<Function<'a>>> for SymbolValueItem<'a> {
    fn from(func: Id<Function<'a>>) -> Self {
        Self::FuncDecl(func)
    }
}

impl<'a> From<Id<VarDecl<'a>>> for SymbolValueItem<'a> {
    fn from(var: Id<VarDecl<'a>>) -> Self {
        Self::VarDecl(var)
    }
}

impl<'a> Identifiable<'a> for SymbolValueItem<'a> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a> {
        match self {
            SymbolValueItem::BuiltinType(builtin) => *builtin,
            SymbolValueItem::VarDecl(var) => ctx.nodes.var_decls[*var].name(ctx),
            SymbolValueItem::FuncDecl(func) => ctx.nodes.funcs[*func].name(ctx),
            SymbolValueItem::FuncArg(arg) => ctx.nodes.func_args[*arg].name(ctx),
            SymbolValueItem::StructDecl(st) => ctx.nodes.st_decls[*st].name(ctx),
            SymbolValueItem::StructAttr(attr) => ctx.nodes.st_attrs[*attr].name(ctx),
            SymbolValueItem::StructInit(st_init) => ctx.nodes.st_inits[*st_init].name(ctx),
            SymbolValueItem::EnumDecl(enm) => ctx.nodes.enms[*enm].name(ctx),
        }
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

// impl<'a> Typed<'a> for SymbolValue<'a> {
//     fn eval_type(
//         &self,
//         symbols: &mut symbol_table_zipper::SymbolTableZipper<'a>,
//     ) -> Result<crate::ir::node::type_signature::TypeSignature<'a>, TypeEvalError<'a>> {
//         match self {
//             SymbolValue::BuiltinType(builtin) => Ok(TypeSignature::Base(builtin.clone())),
//             SymbolValue::VarDecl(var) => var.eval_type(symbols),
//             SymbolValue::FuncDecl(decl) => decl.eval_type(symbols),
//             SymbolValue::FuncArg(arg) => arg.eval_type(symbols),
//             SymbolValue::StructDecl(st) => st.eval_type(symbols),
//             SymbolValue::StructAttr(attr) => attr.eval_type(symbols),
//             SymbolValue::StructInit(st_init) => st_init.eval_type(symbols),
//             SymbolValue::EnumDecl(enm) => enm.eval_type(symbols),
//         }
//     }

//     fn specified_type(&self) -> Option<TypeSignature<'a>> {
//         match self {
//             SymbolValue::BuiltinType(_) => None,
//             SymbolValue::VarDecl(var) => var.specified_type(),
//             SymbolValue::FuncDecl(decl) => decl.specified_type(),
//             SymbolValue::FuncArg(arg) => arg.specified_type(),
//             SymbolValue::StructDecl(st) => st.specified_type(),
//             SymbolValue::StructAttr(attr) => attr.specified_type(),
//             SymbolValue::StructInit(st_init) => st_init.specified_type(),
//             SymbolValue::EnumDecl(enm) => enm.specified_type(),
//         }
//     }

//     fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
//         match self {
//             SymbolValue::BuiltinType(_) => Ok(()),
//             SymbolValue::VarDecl(var) => var.specify_type(new_type),
//             SymbolValue::FuncDecl(decl) => decl.specify_type(new_type),
//             SymbolValue::FuncArg(arg) => arg.specify_type(new_type),
//             SymbolValue::StructDecl(st) => st.specify_type(new_type),
//             SymbolValue::StructAttr(attr) => attr.specify_type(new_type),
//             SymbolValue::StructInit(st_init) => st_init.specify_type(new_type),
//             SymbolValue::EnumDecl(enm) => enm.specify_type(new_type),
//         }
//     }
// }

impl<'a> SymbolTable<'a> {
    pub fn insert(
        &mut self,
        ctx: &mut IrCtx<'a>,
        val: SymbolValueItem<'a>,
    ) -> Result<SymbolValue<'a>, SymbolsError<'a>> {
        let new_sym = ctx.make_symbol(val);
        let val = &ctx.symbols[new_sym];
        if val.is_order_dependent() {
            self.ordered_symbols.push_back(new_sym);
        } else {
            let key: Ident<'a> = val.name(ctx);
            self.scope_global_table
                .try_insert(key, new_sym)
                .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(key))?;
        }
        Ok(new_sym)
    }

    pub fn insert_scope(
        &mut self,
        ident: Ident<'a>,
        scope: SymbolTable<'a>,
    ) -> Result<&mut SymbolTable<'a>, SymbolsError<'a>> {
        let error_ident = ident.clone();

        self.scopes
            .try_insert(ident, scope)
            .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(error_ident))
    }

    pub fn remove_scope(&mut self, ident: Ident<'a>) -> Option<SymbolTable<'a>> {
        self.scopes.remove(&ident)
    }

    pub fn lookup_global_table(&self, ident: Ident<'a>) -> Option<&SymbolValue<'a>> {
        self.scope_global_table.get(&ident)
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
