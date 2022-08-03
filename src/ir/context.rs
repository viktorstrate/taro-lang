use std::collections::HashSet;

use typed_arena::Arena;

use crate::ast;

use super::node::{identifier::Ident, type_signature::TypeSignature};

pub struct IrCtx<'a, 'ctx> {
    types: HashSet<TypeSignature<'a, 'ctx>>,
    identifiers: Arena<Ident<'a>>,
}

impl<'a, 'ctx> IrCtx<'a, 'ctx> {
    fn make_type_sig(
        &'ctx mut self,
        type_sig: TypeSignature<'a, 'ctx>,
    ) -> &'ctx TypeSignature<'a, 'ctx> {
        self.types.get_or_insert(type_sig)
    }

    fn make_ident(&'ctx mut self, ident: ast::node::identifier::Ident<'a>) -> &'ctx Ident<'a> {
        self.identifiers.alloc(Ident::Named {
            def_span: ident.span,
            name: ident.value,
        })
    }

    fn make_anon_ident(&'ctx mut self) -> &'ctx Ident<'a> {
        self.identifiers.alloc(Ident::Anonymous)
    }
}
