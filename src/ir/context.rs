use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use id_arena::Arena;

use crate::{
    ast,
    symbols::symbol_table::{SymbolValue, SymbolValueItem},
};

use super::node::{
    identifier::{Ident, IdentParent, IdentValue, ResolvedIdentValue},
    type_signature::{BuiltinType, TypeSignature, TypeSignatureValue, BUILTIN_TYPES},
    IrNodeArena, NodeRef,
};

pub struct IrCtx<'a> {
    pub types: Arena<TypeSignatureValue<'a>>,
    types_lookup: HashMap<TypeSignatureValue<'a>, TypeSignature<'a>>,
    builtin_types_lookup: HashMap<BuiltinType, TypeSignature<'a>>,
    pub idents: Arena<IdentValue<'a>>,
    pub nodes: IrNodeArena<'a>,
    pub symbols: Arena<SymbolValueItem<'a>>,
}

pub trait IrArenaType<'a>
where
    Self: Sized,
{
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self>;
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self>;
}

impl<'a, T> Index<NodeRef<'a, T>> for IrCtx<'a>
where
    T: IrArenaType<'a>,
{
    type Output = T;

    #[inline]
    fn index(&self, index: NodeRef<'a, T>) -> &Self::Output {
        &T::arena(self)[index.into()]
    }
}

impl<'a, T> IndexMut<NodeRef<'a, T>> for IrCtx<'a>
where
    T: IrArenaType<'a>,
{
    #[inline]
    fn index_mut(&mut self, index: NodeRef<'a, T>) -> &mut Self::Output {
        &mut T::arena_mut(self)[index.into()]
    }
}

impl<'a> Index<TypeSignature<'a>> for IrCtx<'a> {
    type Output = TypeSignatureValue<'a>;

    #[inline]
    fn index(&self, index: TypeSignature<'a>) -> &Self::Output {
        &self.types[index.into()]
    }
}

impl<'a> IndexMut<TypeSignature<'a>> for IrCtx<'a> {
    #[inline]
    fn index_mut(&mut self, index: TypeSignature<'a>) -> &mut Self::Output {
        &mut self.types[index.into()]
    }
}

impl<'a> Index<Ident<'a>> for IrCtx<'a> {
    type Output = IdentValue<'a>;

    #[inline]
    fn index(&self, index: Ident<'a>) -> &Self::Output {
        &self.idents[index.into()]
    }
}

impl<'a> IndexMut<Ident<'a>> for IrCtx<'a> {
    #[inline]
    fn index_mut(&mut self, index: Ident<'a>) -> &mut Self::Output {
        &mut self.idents[index.into()]
    }
}

impl<'a> Index<SymbolValue<'a>> for IrCtx<'a> {
    type Output = SymbolValueItem<'a>;

    #[inline]
    fn index(&self, index: SymbolValue<'a>) -> &Self::Output {
        &self.symbols[index.into()]
    }
}

impl<'a> IndexMut<SymbolValue<'a>> for IrCtx<'a> {
    #[inline]
    fn index_mut(&mut self, index: SymbolValue<'a>) -> &mut Self::Output {
        &mut self.symbols[index.into()]
    }
}

impl<'a> IrCtx<'a> {
    pub fn new() -> Self {
        let mut types: Arena<TypeSignatureValue<'a>> = Arena::new();
        let builtin_types_lookup = BUILTIN_TYPES
            .iter()
            .map(|&builtin| {
                (
                    builtin,
                    types.alloc(TypeSignatureValue::Builtin(builtin)).into(),
                )
            })
            .collect();

        IrCtx {
            types,
            types_lookup: HashMap::new(),
            builtin_types_lookup,
            idents: Arena::new(),
            nodes: IrNodeArena::new(),
            symbols: Arena::new(),
        }
    }

    pub fn get_type_sig(&mut self, type_sig: TypeSignatureValue<'a>) -> TypeSignature<'a> {
        match type_sig {
            TypeSignatureValue::Builtin(builtin) => return self.get_builtin_type_sig(builtin),
            TypeSignatureValue::TypeVariable(_) => {
                panic!("cannot get type signature from type variable")
            }
            _ => {}
        }

        if let Some(found_type) = self.types_lookup.get(&type_sig) {
            return *found_type;
        }

        let type_sig_id = self.types.alloc(type_sig.clone()).into();
        self.types_lookup.insert(type_sig, type_sig_id);

        type_sig_id
    }

    pub fn make_type_var(&mut self) -> TypeSignature<'a> {
        self.types
            .alloc_with_id(|id| TypeSignatureValue::TypeVariable(id))
            .into()
    }

    pub fn get_builtin_type_sig(&self, builtin: BuiltinType) -> TypeSignature<'a> {
        *self
            .builtin_types_lookup
            .get(&builtin)
            .expect("get builtin type signature")
    }

    pub fn make_ident(
        &mut self,
        ident: ast::node::identifier::Ident<'a>,
        parent: IdentParent<'a>,
    ) -> Ident<'a> {
        Ident {
            id: self
                .idents
                .alloc(IdentValue::Resolved(ResolvedIdentValue::Named {
                    def_span: ident.span,
                    name: ident.value,
                })),
            parent,
        }
    }

    pub fn make_builtin_ident(&mut self, builtin: BuiltinType) -> Ident<'a> {
        Ident {
            id: self
                .idents
                .alloc(IdentValue::Resolved(ResolvedIdentValue::BuiltinType(
                    builtin,
                ))),
            parent: IdentParent::BuiltinIdent,
        }
    }

    pub fn make_anon_ident(&mut self, parent: IdentParent<'a>) -> Ident<'a> {
        Ident {
            id: self
                .idents
                .alloc(IdentValue::Resolved(ResolvedIdentValue::Anonymous)),
            parent,
        }
    }

    pub fn make_unresolved_ident(&mut self, ident: ast::node::identifier::Ident<'a>) -> Ident<'a> {
        Ident {
            id: self.idents.alloc(IdentValue::Unresolved(ident)),
            parent: IdentParent::UnresolvedIdent,
        }
    }

    pub fn make_symbol(&mut self, symbol: SymbolValueItem<'a>) -> SymbolValue<'a> {
        self.symbols.alloc(symbol).into()
    }
}
