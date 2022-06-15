use std::collections::{HashMap, VecDeque};

use crate::ast::node::{
    function::{Function, FunctionArg},
    identifier::{Ident, Identifiable},
    statement::VarDecl,
    structure::{Struct, StructAttr},
    type_signature::{TypeEvalError, TypeSignature, Typed},
};

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

/// A value returned from a symbol lookup
#[derive(Debug, Clone)]
pub enum SymbolValue<'a> {
    BuiltinType(Ident<'a>),
    VarDecl(VarDecl<'a>),
    FuncDecl(Function<'a>),
    FuncArg(FunctionArg<'a>),
    StructDecl(Struct<'a>),
    StructAttr(StructAttr<'a>),
}

impl<'a> From<Function<'a>> for SymbolValue<'a> {
    fn from(func: Function<'a>) -> Self {
        Self::FuncDecl(func)
    }
}

impl<'a> From<VarDecl<'a>> for SymbolValue<'a> {
    fn from(var: VarDecl<'a>) -> Self {
        Self::VarDecl(var)
    }
}

impl<'a> Identifiable<'a> for SymbolValue<'a> {
    fn name(&self) -> &Ident<'a> {
        match self {
            SymbolValue::BuiltinType(builtin) => builtin,
            SymbolValue::VarDecl(var) => var.name(),
            SymbolValue::FuncDecl(func) => func.name(),
            SymbolValue::FuncArg(arg) => arg.name(),
            SymbolValue::StructDecl(st) => st.name(),
            SymbolValue::StructAttr(attr) => attr.name(),
        }
    }
}

impl SymbolValue<'_> {
    fn is_order_dependent(&self) -> bool {
        match self {
            SymbolValue::VarDecl(_) => true,
            _ => false,
        }
    }
}

impl<'a> Typed<'a> for SymbolValue<'a> {
    fn eval_type(
        &self,
        symbols: &mut super::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<crate::ast::node::type_signature::TypeSignature<'a>, TypeEvalError<'a>> {
        match self {
            SymbolValue::BuiltinType(builtin) => Ok(TypeSignature::Base(builtin.clone())),
            SymbolValue::VarDecl(var) => var.eval_type(symbols),
            SymbolValue::FuncDecl(func) => func.eval_type(symbols),
            SymbolValue::FuncArg(arg) => arg.eval_type(symbols),
            SymbolValue::StructDecl(st) => st.eval_type(symbols),
            SymbolValue::StructAttr(attr) => attr.eval_type(symbols),
        }
    }

    fn specified_type(&self) -> Option<&TypeSignature<'a>> {
        match self {
            SymbolValue::BuiltinType(_) => None,
            SymbolValue::VarDecl(var) => var.specified_type(),
            SymbolValue::FuncDecl(func) => func.specified_type(),
            SymbolValue::FuncArg(arg) => arg.specified_type(),
            SymbolValue::StructDecl(st) => st.specified_type(),
            SymbolValue::StructAttr(attr) => attr.specified_type(),
        }
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) {
        match self {
            SymbolValue::BuiltinType(_) => {}
            SymbolValue::VarDecl(var) => var.specify_type(new_type),
            SymbolValue::FuncDecl(func) => func.specify_type(new_type),
            SymbolValue::FuncArg(arg) => arg.specify_type(new_type),
            SymbolValue::StructDecl(st) => st.specify_type(new_type),
            SymbolValue::StructAttr(attr) => attr.specify_type(new_type),
        }
    }
}

impl<'a> SymbolTable<'a> {
    pub fn insert(
        &mut self,
        val: SymbolValue<'a>,
    ) -> Result<&mut SymbolValue<'a>, SymbolsError<'a>> {
        if val.is_order_dependent() {
            self.ordered_symbols.push_back(val);
            Ok(self.ordered_symbols.back_mut().expect("was just added"))
        } else {
            let key: Ident<'a> = val.name().clone();
            let error_ident = key.clone();
            self.scope_global_table
                .try_insert(key, val)
                .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(error_ident))
        }
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

    pub fn remove_scope(&mut self, ident: &Ident<'a>) -> Option<SymbolTable<'a>> {
        self.scopes.remove(ident)
    }

    pub fn pop_ordered_symbol(&mut self) -> Option<SymbolValue<'a>> {
        self.ordered_symbols.pop_front()
    }

    pub fn lookup_global_table(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
        self.scope_global_table.get(ident)
    }
}
