use std::collections::{HashMap, VecDeque};

use crate::ir::node::{
    enumeration::Enum,
    function::{Function, FunctionArg},
    identifier::{Ident, Identifiable},
    statement::VarDecl,
    structure::{Struct, StructAttr, StructInit},
    type_signature::{BuiltinType, TypeEvalError, TypeSignature},
};

pub mod symbol_table_zipper;

#[derive(Debug)]
pub enum SymbolsError<'a, 'ctx> {
    SymbolAlreadyExistsInScope(Ident<'a, 'ctx>),
    ScopeNotFound(Ident<'a, 'ctx>),
    MovePastGlobalScope,
}

#[derive(Default, Debug)]
pub struct SymbolTable<'a, 'ctx> {
    /// Symbols that are available from the entire scope
    pub(super) scope_global_table: HashMap<Ident<'a, 'ctx>, SymbolValue<'a, 'ctx>>,
    /// An list for order dependent symbols such as variable declarations,
    /// where symbols are first available after they have been declared.
    pub(super) ordered_symbols: VecDeque<SymbolValue<'a, 'ctx>>,
    /// Nested scopes such as function bodies and struct definitions.
    pub(super) scopes: HashMap<Ident<'a, 'ctx>, SymbolTable<'a, 'ctx>>,
}

/// A value returned from a symbol lookup
#[derive(Debug)]
pub enum SymbolValue<'a, 'ctx> {
    BuiltinType(Ident<'a, 'ctx>),
    VarDecl(&'ctx VarDecl<'a, 'ctx>),
    FuncDecl(&'ctx Function<'a, 'ctx>),
    FuncArg(&'ctx FunctionArg<'a, 'ctx>),
    StructDecl(&'ctx Struct<'a, 'ctx>),
    StructAttr(&'ctx StructAttr<'a, 'ctx>),
    StructInit(&'ctx StructInit<'a, 'ctx>),
    EnumDecl(&'ctx Enum<'a, 'ctx>),
}

impl<'a, 'ctx> From<&'ctx mut Function<'a, 'ctx>> for SymbolValue<'a, 'ctx> {
    fn from(func: &'ctx mut Function<'a, 'ctx>) -> Self {
        Self::FuncDecl(func)
    }
}

impl<'a, 'ctx> From<&'ctx mut VarDecl<'a, 'ctx>> for SymbolValue<'a, 'ctx> {
    fn from(var: &'ctx mut VarDecl<'a, 'ctx>) -> Self {
        Self::VarDecl(var)
    }
}

impl<'a, 'ctx> Identifiable<'a, 'ctx> for SymbolValue<'a, 'ctx> {
    fn name(&self) -> &Ident<'a, 'ctx> {
        match self {
            SymbolValue::BuiltinType(builtin) => builtin,
            SymbolValue::VarDecl(var) => var.name(),
            SymbolValue::FuncDecl(func) => func.name(),
            SymbolValue::FuncArg(arg) => arg.name(),
            SymbolValue::StructDecl(st) => st.name(),
            SymbolValue::StructAttr(attr) => attr.name(),
            SymbolValue::StructInit(st_init) => st_init.name(),
            SymbolValue::EnumDecl(enm) => enm.name(),
        }
    }
}

impl SymbolValue<'_, '_> {
    fn is_order_dependent(&self) -> bool {
        match self {
            SymbolValue::VarDecl(_) => true,
            _ => false,
        }
    }
}

// impl<'a> Typed<'a> for SymbolValue<'a, 'ctx> {
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

impl<'a, 'ctx> SymbolTable<'a, 'ctx> {
    pub fn insert(
        &mut self,
        val: SymbolValue<'a, 'ctx>,
    ) -> Result<&mut SymbolValue<'a, 'ctx>, SymbolsError<'a, 'ctx>> {
        if val.is_order_dependent() {
            self.ordered_symbols.push_back(val);
            Ok(self.ordered_symbols.back_mut().expect("was just added"))
        } else {
            let key: Ident<'a, 'ctx> = val.name().clone();
            let error_ident = key.clone();
            self.scope_global_table
                .try_insert(key, val)
                .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(error_ident))
        }
    }

    pub fn insert_scope(
        &mut self,
        ident: Ident<'a, 'ctx>,
        scope: SymbolTable<'a, 'ctx>,
    ) -> Result<&'ctx mut SymbolTable<'a, 'ctx>, SymbolsError<'a, 'ctx>> {
        let error_ident = ident.clone();

        self.scopes
            .try_insert(ident, scope)
            .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(error_ident))
    }

    pub fn remove_scope(&mut self, ident: &Ident<'a, 'ctx>) -> Option<SymbolTable<'a, 'ctx>> {
        self.scopes.remove(ident)
    }

    pub fn lookup_global_table(&self, ident: &Ident<'a, 'ctx>) -> Option<&SymbolValue<'a, 'ctx>> {
        self.scope_global_table.get(ident)
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
