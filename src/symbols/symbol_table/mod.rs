use std::collections::{HashMap, VecDeque};

use crate::ast::node::{
    function::{Function, FunctionArg},
    identifier::{Ident, Identifiable},
    statement::VarDecl,
    structure::{Struct, StructAttr},
    type_signature::{TypeEvalError, TypeSignature, Typed},
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
        symbols: &mut symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<crate::ast::node::type_signature::TypeSignature<'a>, TypeEvalError<'a>> {
        match self {
            SymbolValue::BuiltinType(builtin) => Ok(TypeSignature::Base(builtin.clone())),
            SymbolValue::VarDecl(var) => var.eval_type(symbols),
            SymbolValue::FuncDecl(decl) => decl.eval_type(symbols),
            SymbolValue::FuncArg(arg) => arg.eval_type(symbols),
            SymbolValue::StructDecl(st) => st.eval_type(symbols),
            SymbolValue::StructAttr(attr) => attr.eval_type(symbols),
        }
    }

    fn specified_type(&self) -> Option<&TypeSignature<'a>> {
        match self {
            SymbolValue::BuiltinType(_) => None,
            SymbolValue::VarDecl(var) => var.specified_type(),
            SymbolValue::FuncDecl(decl) => decl.specified_type(),
            SymbolValue::FuncArg(arg) => arg.specified_type(),
            SymbolValue::StructDecl(st) => st.specified_type(),
            SymbolValue::StructAttr(attr) => attr.specified_type(),
        }
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) {
        match self {
            SymbolValue::BuiltinType(_) => {}
            SymbolValue::VarDecl(var) => var.specify_type(new_type),
            SymbolValue::FuncDecl(decl) => decl.specify_type(new_type),
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

    pub fn lookup_global_table(&self, ident: &Ident<'a>) -> Option<&SymbolValue<'a>> {
        self.scope_global_table.get(ident)
    }

    // fn lookup_nested_ident_scope(&self, ident: &'a Box<Ident<'a>>) -> Option<&SymbolTable<'a>> {
    //     match ident.value {
    //         IdentValue::Nested { scope, name: _ } => self
    //             .lookup_nested_ident_scope(ident)
    //             .and_then(|inner_scope| inner_scope.scopes.get(ident)),
    //         _ => self.scopes.get(ident),
    //     }
    // }

    // pub fn lookup_global_table(&self, ident: &'a Ident<'a>) -> Option<&SymbolValue<'a>> {
    //     match &ident.value {
    //         IdentValue::Nested { scope, name } => self
    //             .lookup_nested_ident_scope(scope)
    //             .and_then(|scope| scope.scope_global_table.get(ident)),
    //         _ => self.scope_global_table.get(ident),
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::{ast_walker::walk_ast, node::expression::Expr},
        parser::parse_ast,
        symbols::symbol_walker::SymbolCollector,
    };

    use super::*;

    #[test]
    fn test_locate_ordered_symbol() {
        let mut ast = parse_ast("let x: Boolean = true").unwrap();
        let mut collector = SymbolCollector::default();
        let symtable = walk_ast(&mut collector, &mut ast).unwrap();
        let sym_val = symtable.ordered_symbols.front().unwrap();

        assert_eq!(*sym_val.name(), Ident::new_unplaced("x"));
        match sym_val {
            SymbolValue::VarDecl(var_decl) => {
                assert_matches!(var_decl.value, Expr::BoolLiteral(true));
            }
            _ => assert!(false),
        }
    }
}
