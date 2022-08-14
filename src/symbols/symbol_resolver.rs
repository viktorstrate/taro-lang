use crate::ir::{
    context::IrCtx,
    ir_walker::{IrWalker, ScopeValue},
    node::identifier::{Ident, IdentParent, IdentValue, Identifiable},
};

use super::symbol_table::{
    symbol_table_zipper::SymbolTableZipper, SymbolTable, SymbolValueItem, SymbolsError,
};

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

    fn visit_ordered_symbol(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        self.symbols.visit_next_symbol(ctx);
        Ok(())
    }

    fn visit_ident(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        parent: IdentParent<'a>,
        ident: Ident<'a>,
    ) -> Result<Ident<'a>, Self::Error> {
        let resolved_ident = match &ctx[ident] {
            IdentValue::Unresolved(_) => {
                println!("Visit unresolved ident: {:?} {:?}", ctx[ident], parent);
                match parent {
                    IdentParent::StructInitValueName(st_init) => {
                        println!("STRUCT INIT VALUE NAME {:?}", ctx[st_init]);
                        let st_name = ctx[ctx[st_init].parent].struct_name;

                        let st_sym_val = *self
                            .symbols
                            .lookup(ctx, st_name)
                            .ok_or(SymbolsError::ScopeNotFound(ident))?;

                        println!("Found struct sym {:?}", ctx[st_sym_val]);

                        let st = match ctx[st_sym_val] {
                            SymbolValueItem::StructDecl(st) => st,
                            _ => unreachable!("expected to find struct"),
                        };

                        println!(
                            "Found struct {:?} {:?}",
                            ctx[st], ctx[ctx[ctx[st].attrs[0]].name]
                        );

                        let attr = st
                            .lookup_attr(ident, ctx)
                            .ok_or(SymbolsError::ScopeNotFound(ident))?;

                        println!("Found struct attr {:?}", ctx[attr]);
                        Some(ctx[attr].name)
                    }
                    IdentParent::StructAccessAttrName(st_access) => {
                        let st_attr = st_access
                            .lookup_attr_chain(ctx, &mut self.symbols)
                            .map_err(|_| SymbolsError::ScopeNotFound(ident))?[0];

                        Some(ctx[st_attr].name)
                    }
                    _ => {
                        let sym_id = *self
                            .symbols
                            .lookup(ctx, ident)
                            .ok_or(SymbolsError::ScopeNotFound(ident))?;

                        let sym = *&ctx[sym_id];
                        Some(sym.name(ctx))
                    }
                }
            }
            IdentValue::Resolved(_) => None,
        };

        if let Some(sym_ident) = resolved_ident {
            debug_assert!(matches!(ctx[sym_ident], IdentValue::Resolved(_)));

            Ok(sym_ident)
        } else {
            Ok(ident)
        }
    }
}
