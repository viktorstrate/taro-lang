use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Index, IndexMut},
    rc::Rc,
};

use id_arena::{Arena, Id};

use crate::{
    ast,
    symbols::symbol_table::{SymbolValue, SymbolValueItem},
};

use super::{
    late_init::LateInit,
    node::{
        identifier::{Ident, IdentParent, IdentValue, ResolvedIdentValue},
        type_signature::{
            BuiltinType, TypeSignature, TypeSignatureContext, TypeSignatureParent,
            TypeSignatureValue, BUILTIN_TYPES,
        },
        IrNodeArena, NodeRef,
    },
};

pub struct IrCtx<'a> {
    pub types: Arena<TypeSignatureValue<'a>>,
    types_lookup: HashMap<TypeSignatureValue<'a>, Id<TypeSignatureValue<'a>>>,
    builtin_types_lookup: HashMap<BuiltinType, Id<TypeSignatureValue<'a>>>,
    pub idents: Arena<IdentValue<'a>>,
    pub nodes: IrNodeArena<'a>,
    pub symbols: Arena<SymbolValueItem<'a>>,
}

impl<'a> Debug for IrCtx<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IrCtx").finish()
    }
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

impl<'a> Index<&TypeSignature<'a>> for IrCtx<'a> {
    type Output = TypeSignatureValue<'a>;

    #[inline]
    fn index(&self, index: &TypeSignature<'a>) -> &Self::Output {
        &self.types[index.into()]
    }
}

impl<'a> IndexMut<&TypeSignature<'a>> for IrCtx<'a> {
    #[inline]
    fn index_mut(&mut self, index: &TypeSignature<'a>) -> &mut Self::Output {
        &mut self.types[(index).into()]
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
            .map(|&builtin| (builtin, types.alloc(TypeSignatureValue::Builtin(builtin))))
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

    pub fn get_type_sig(
        &mut self,
        type_sig: TypeSignatureValue<'a>,
        type_ctx: Rc<TypeSignatureContext<'a>>,
    ) -> TypeSignature<'a> {
        let result = match type_sig {
            TypeSignatureValue::Builtin(builtin) => Some(self.get_builtin_type_sig(builtin).id),
            TypeSignatureValue::TypeVariable(_) => {
                panic!("cannot get type signature from type variable")
            }
            _ => None,
        };

        let result = result.or_else(|| self.types_lookup.get(&type_sig).cloned());

        let result = result.unwrap_or_else(|| {
            let type_sig_id = self.types.alloc(type_sig.clone()).into();
            self.types_lookup.insert(type_sig, type_sig_id);

            type_sig_id
        });

        TypeSignature {
            id: result,
            context: type_ctx,
        }
    }

    pub fn make_type_var(&mut self, parent: TypeSignatureParent<'a>) -> TypeSignature<'a> {
        let id = self
            .types
            .alloc_with_id(|id| TypeSignatureValue::TypeVariable(id));

        TypeSignature {
            id,
            context: TypeSignatureContext {
                parent,
                type_span: None,
            }
            .alloc(),
        }
    }

    pub fn get_builtin_type_sig(&self, builtin: BuiltinType) -> TypeSignature<'a> {
        let type_id = *self
            .builtin_types_lookup
            .get(&builtin)
            .expect("get builtin type signature");

        TypeSignature {
            id: type_id,
            context: TypeSignatureContext {
                parent: TypeSignatureParent::Builtin,
                type_span: None,
            }
            .alloc(),
        }
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
            parent: parent.into(),
        }
    }

    pub fn make_builtin_ident(&mut self, builtin: BuiltinType) -> Ident<'a> {
        Ident {
            id: self
                .idents
                .alloc(IdentValue::Resolved(ResolvedIdentValue::BuiltinType(
                    builtin,
                ))),
            parent: IdentParent::BuiltinIdent.into(),
        }
    }

    pub fn make_anon_ident(&mut self, parent: IdentParent<'a>) -> Ident<'a> {
        Ident {
            id: self
                .idents
                .alloc(IdentValue::Resolved(ResolvedIdentValue::Anonymous)),
            parent: parent.into(),
        }
    }

    pub fn make_unresolved_ident(
        &mut self,
        ident: ast::node::identifier::Ident<'a>,
        parent: LateInit<IdentParent<'a>>,
    ) -> Ident<'a> {
        Ident {
            id: self.idents.alloc(IdentValue::Unresolved(ident)),
            parent,
        }
    }

    pub fn make_symbol(&mut self, symbol: SymbolValueItem<'a>) -> SymbolValue<'a> {
        self.symbols.alloc(symbol).into()
    }
}
