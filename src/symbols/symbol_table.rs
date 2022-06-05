use std::collections::HashMap;

use crate::ast::node::{
    expression::ExprValueError,
    function::{Function, FunctionArg},
    identifier::{Ident, Identifiable},
    statement::VarDecl,
    structure::{Struct, StructAttr},
    type_signature::{TypeSignature, Typed},
};

#[derive(Debug)]
pub enum SymbolsError<'a> {
    SymbolAlreadyExistsInScope(Ident<'a>),
    ScopeNotFound(Ident<'a>),
    MovePastGlobalScope,
}

#[derive(Default, Debug)]
pub struct SymbolTable<'a> {
    table: HashMap<Ident<'a>, SymbolValue<'a>>,
    scopes: HashMap<Ident<'a>, SymbolTable<'a>>,
}

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

impl<'a> Typed<'a> for SymbolValue<'a> {
    type Error = ExprValueError<'a>;

    fn type_sig(
        &self,
        symbols: &mut super::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<crate::ast::node::type_signature::TypeSignature<'a>, Self::Error> {
        match self {
            SymbolValue::BuiltinType(builtin) => Ok(TypeSignature::Base(builtin.clone())),
            SymbolValue::VarDecl(var) => var.type_sig(symbols),
            SymbolValue::FuncDecl(func) => {
                func.type_sig(symbols).map_err(ExprValueError::FunctionType)
            }
            SymbolValue::FuncArg(arg) => {
                arg.type_sig(symbols).map_err(ExprValueError::FunctionType)
            }
            SymbolValue::StructDecl(st) => st.type_sig(symbols),
            SymbolValue::StructAttr(attr) => attr.type_sig(symbols),
        }
    }
}

impl<'a> SymbolTable<'a> {
    pub fn insert(
        &mut self,
        val: SymbolValue<'a>,
    ) -> Result<&mut SymbolValue<'a>, SymbolsError<'a>> {
        let key: Ident<'a> = val.name().clone();
        let error_ident = key.clone();

        self.table
            .try_insert(key, val)
            .map_err(move |_| SymbolsError::SymbolAlreadyExistsInScope(error_ident))
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

    pub fn locate(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
        self.table.get(ident)
    }
}
