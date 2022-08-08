use crate::ir::{
    context::IrCtx,
    ir_walker::{IrWalker, ScopeValue},
    node::identifier::{Ident, IdentValue, Identifiable},
};

use super::symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolTable, SymbolsError};

pub struct SymbolResolver<'a> {
    pub symbols: SymbolTableZipper<'a>,
}

impl<'a> SymbolResolver<'a> {
    pub fn new(symbols: SymbolTable<'a>) -> Self {
        Self {
            symbols: symbols.into(),
        }
    }
}

impl<'a> IrWalker<'a> for SymbolResolver<'a> {
    type Scope = ();
    type Error = SymbolsError<'a>;

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), SymbolsError<'a>> {
        value.visit_scope_begin(ctx, &mut self.symbols);
        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        _child: Self::Scope,
        _value: ScopeValue<'a>,
    ) -> Result<(), SymbolsError<'a>> {
        self.symbols
            .exit_scope(ctx)
            .expect("scope should not be global scope");

        Ok(())
    }

    fn visit_ident(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        ident: Ident<'a>,
    ) -> Result<Ident<'a>, Self::Error> {
        let sym = match &ctx[ident] {
            IdentValue::Unresolved(_) => {
                let sym_id = *self
                    .symbols
                    .lookup(ctx, ident)
                    .ok_or(SymbolsError::ScopeNotFound(ident))?;

                let sym = *&ctx[sym_id];
                Some(sym)
            }
            IdentValue::Resolved(_) => None,
        };

        if let Some(sym_val) = sym {
            let sym_ident = sym_val.name(ctx);
            debug_assert!(matches!(ctx[sym_ident], IdentValue::Resolved(_)));

            Ok(sym_ident)
        } else {
            Ok(ident)
        }
    }
}
