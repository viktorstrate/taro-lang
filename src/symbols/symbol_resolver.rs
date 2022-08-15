use crate::ir::{
    context::IrCtx,
    ir_walker::{IrWalker, ScopeValue},
    node::{
        identifier::{Ident, IdentParent, IdentValue, Identifiable},
        type_signature::{TypeEvalError, TypeSignatureValue, Typed},
    },
};

use super::symbol_table::{
    symbol_table_zipper::SymbolTableZipper, SymbolTable, SymbolValue, SymbolValueItem,
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

#[derive(Debug)]
pub enum SymbolResolutionError<'a> {
    UnknownIdentifier(Ident<'a>),
    TypeEval(TypeEvalError<'a>),
    InitNonEnum(SymbolValue<'a>),
    UnknownEnumValue {
        enum_name: Ident<'a>,
        enum_value: Ident<'a>,
    },
}

impl<'a> IrWalker<'a> for SymbolResolver<'a> {
    type Scope = ();
    type Error = SymbolResolutionError<'a>;

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), Self::Error> {
        value.visit_scope_begin(ctx, &mut self.symbols);
        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        _child: Self::Scope,
        _value: ScopeValue<'a>,
    ) -> Result<(), Self::Error> {
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
        println!("Resolve ident {parent:?}");
        let resolved_ident = match &ctx[ident] {
            IdentValue::Unresolved(_) => match parent {
                IdentParent::StructInitValueName(st_init) => {
                    let st_name = ctx[ctx[st_init].parent].struct_name;

                    let st_sym_val = *self
                        .symbols
                        .lookup(ctx, st_name)
                        .ok_or(SymbolResolutionError::UnknownIdentifier(st_name))?;

                    let st = match ctx[st_sym_val] {
                        SymbolValueItem::StructDecl(st) => st,
                        _ => unreachable!("expected to find struct"),
                    };

                    let attr = st
                        .lookup_attr(ident, ctx)
                        .ok_or(SymbolResolutionError::UnknownIdentifier(ident))?;

                    Some(ctx[attr].name)
                }
                IdentParent::StructAccessAttrName(st_access) => {
                    let st_attr = st_access
                        .lookup_attr(ctx, &mut self.symbols)
                        .map_err(|_| SymbolResolutionError::UnknownIdentifier(ident))?;

                    match ctx[st_attr].type_sig {
                        Some(type_sig) => match ctx[type_sig] {
                            TypeSignatureValue::Unresolved(type_ident) => {
                                let resolved_type_sig = self
                                    .symbols
                                    .lookup(ctx, type_ident)
                                    .ok_or(SymbolResolutionError::UnknownIdentifier(type_ident))?
                                    .clone()
                                    .eval_type(&mut self.symbols, ctx)
                                    .map_err(SymbolResolutionError::TypeEval)?;

                                ctx[st_attr].type_sig = Some(resolved_type_sig);
                            }
                            _ => {}
                        },
                        _ => {}
                    }

                    Some(ctx[st_attr].name)
                }
                IdentParent::EnumInitValueName(enm_init) => {
                    let enm_name = ctx[enm_init]
                        .enum_name
                        .expect("TODO: handle type inference for enums");

                    println!("Lookup enm name");

                    let enm_sym = *self
                        .symbols
                        .lookup(ctx, enm_name)
                        .ok_or(SymbolResolutionError::UnknownIdentifier(enm_name))?;

                    println!("Lookup enm name end");

                    let enm = match ctx[enm_sym] {
                        SymbolValueItem::EnumDecl(enm) => Ok(enm),
                        _ => Err(SymbolResolutionError::InitNonEnum(enm_sym)),
                    }?;

                    let (_, enm_val) = enm.lookup_value(ctx, ctx[enm_init].enum_value).ok_or(
                        SymbolResolutionError::UnknownEnumValue {
                            enum_name: enm_name,
                            enum_value: ctx[enm_init].enum_value,
                        },
                    )?;

                    Some(ctx[enm_val].name)
                }
                val => {
                    println!("Lookup other, parent: {val:?}");

                    let sym_id = *self
                        .symbols
                        .lookup(ctx, ident)
                        .ok_or(SymbolResolutionError::UnknownIdentifier(ident))?;

                    println!("Lookup other done");

                    let sym = *&ctx[sym_id];
                    Some(sym.name(ctx))
                }
            },
            IdentValue::Resolved(_) => None,
        };

        println!("Resolve ident done");

        if let Some(sym_ident) = resolved_ident {
            debug_assert!(matches!(ctx[sym_ident], IdentValue::Resolved(_)));

            Ok(sym_ident)
        } else {
            Ok(ident)
        }
    }

    fn visit_type_sig(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        type_sig: crate::ir::node::type_signature::TypeSignature<'a>,
    ) -> Result<crate::ir::node::type_signature::TypeSignature<'a>, Self::Error> {
        let updated_type_sig = match ctx[type_sig] {
            TypeSignatureValue::Unresolved(ident) => {
                let sym_val = *self
                    .symbols
                    .lookup(ctx, ident)
                    .ok_or(SymbolResolutionError::UnknownIdentifier(ident))?;

                let new_type = sym_val
                    .eval_type(&mut self.symbols, ctx)
                    .map_err(SymbolResolutionError::TypeEval)?;

                new_type
            }
            _ => type_sig,
        };

        Ok(updated_type_sig)
    }
}
