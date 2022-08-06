use std::collections::HashMap;

use id_arena::Arena;

use crate::{
    ast,
    symbols::symbol_table::{SymbolValue, SymbolValueItem},
};

use super::node::{
    identifier::{Ident, IdentValue, ResolvedIdentValue},
    type_signature::{BuiltinType, TypeSignature, TypeSignatureValue, BUILTIN_TYPES},
    IrNodeArena,
};

pub struct IrCtx<'a> {
    pub types: Arena<TypeSignatureValue<'a>>,
    types_lookup:
        HashMap<crate::ast::node::type_signature::TypeSignatureValue<'a>, TypeSignature<'a>>,
    builtin_types_lookup: HashMap<BuiltinType, TypeSignature<'a>>,
    pub idents: Arena<IdentValue<'a>>,
    pub nodes: IrNodeArena<'a>,
    pub symbols: Arena<SymbolValueItem<'a>>,
}

impl<'a> IrCtx<'a> {
    pub fn new() -> Self {
        let mut types: Arena<TypeSignatureValue<'a>> = Arena::new();
        let builtin_types_lookup = BUILTIN_TYPES
            .iter()
            .map(|&builtin| (builtin, types.alloc(TypeSignatureValue::Builtin(builtin))))
            .collect();

        IrCtx {
            types: Arena::new(),
            types_lookup: HashMap::new(),
            builtin_types_lookup,
            idents: Arena::new(),
            nodes: IrNodeArena::new(),
            symbols: Arena::new(),
        }
    }

    pub fn get_type_sig(
        &mut self,
        type_sig: crate::ast::node::type_signature::TypeSignature<'a>,
    ) -> TypeSignature<'a> {
        if let Some(found_type) = self.types_lookup.get(&type_sig.value) {
            return *found_type;
        }

        let ir_type = match type_sig.value {
            ast::node::type_signature::TypeSignatureValue::Base(base) => {
                TypeSignatureValue::Unresolved(base)
            }
            ast::node::type_signature::TypeSignatureValue::Function { args, return_type } => {
                TypeSignatureValue::Function {
                    args: args.into_iter().map(|arg| self.get_type_sig(arg)).collect(),
                    return_type: self.get_type_sig(*return_type),
                }
            }
            ast::node::type_signature::TypeSignatureValue::Tuple(types) => {
                TypeSignatureValue::Tuple(types.into_iter().map(|t| self.get_type_sig(t)).collect())
            }
        };

        self.types.alloc(ir_type)
    }

    pub fn get_builtin_type_sig(&self, builtin: BuiltinType) -> TypeSignature<'a> {
        *self
            .builtin_types_lookup
            .get(&builtin)
            .expect("get builtin type signature")
    }

    // pub fn get_resolved_type_sig(&self, ident: Ident<'a>) -> TypeSignature<'a> {
    //     *self
    //         .resolved_types_lookup
    //         .get(&ident)
    //         .expect("get resolved type signature")
    // }

    pub fn make_type_sig(&mut self, type_sig: TypeSignatureValue<'a>) -> TypeSignature<'a> {
        self.types.alloc(type_sig)
    }

    pub fn make_ident(&mut self, ident: ast::node::identifier::Ident<'a>) -> Ident<'a> {
        self.idents
            .alloc(IdentValue::Resolved(ResolvedIdentValue::Named {
                def_span: ident.span,
                name: ident.value,
            }))
    }

    pub fn make_builtin_ident(&mut self, builtin: BuiltinType) -> Ident<'a> {
        self.idents
            .alloc(IdentValue::Resolved(ResolvedIdentValue::BuiltinType(
                builtin,
            )))
    }

    pub fn make_anon_ident(&mut self) -> Ident<'a> {
        self.idents
            .alloc(IdentValue::Resolved(ResolvedIdentValue::Anonymous))
    }

    pub fn make_unresolved_ident(&mut self, ident: ast::node::identifier::Ident<'a>) -> Ident<'a> {
        self.idents.alloc(IdentValue::Unresolved(ident))
    }

    pub fn make_symbol(&mut self, symbol: SymbolValueItem<'a>) -> SymbolValue<'a> {
        self.symbols.alloc(symbol)
    }
}
