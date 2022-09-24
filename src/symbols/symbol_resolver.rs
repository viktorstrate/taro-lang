use std::assert_matches::debug_assert_matches;

use crate::ir::{
    context::IrCtx,
    ir_walker::{walk_expr, IrWalker, ScopeValue},
    node::{
        enumeration::{Enum, EnumInit},
        expression::Expr,
        function::FunctionCall,
        identifier::{Ident, IdentParent, IdentValue, Identifiable},
        structure::StructAccess,
        type_signature::{TypeEvalError, TypeSignatureValue, Typed},
        IrAlloc, NodeRef,
    },
};

use super::symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolTable};

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
    UnknownEnumValue {
        enm: NodeRef<'a, Enum<'a>>,
        // enum_name: Ident<'a>,
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
        ident: Ident<'a>,
    ) -> Result<(), Self::Error> {
        resolve_ident(&mut self.symbols, ctx, ident)?;
        Ok(())
    }

    fn visit_type_sig(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        type_sig: crate::ir::node::type_signature::TypeSignature<'a>,
    ) -> Result<crate::ir::node::type_signature::TypeSignature<'a>, Self::Error> {
        let updated_type_sig = match ctx[&type_sig].clone() {
            TypeSignatureValue::Unresolved(ident) => {
                let sym_val = self
                    .symbols
                    .lookup(ctx, ident)
                    .ok_or(SymbolResolutionError::UnknownIdentifier(ident))?;

                let mut new_type = sym_val
                    .eval_type(&mut self.symbols, ctx)
                    .map_err(SymbolResolutionError::TypeEval)?;

                new_type.context = type_sig.context;

                new_type
            }
            _ => type_sig,
        };

        Ok(updated_type_sig)
    }

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), Self::Error> {
        match ctx[expr] {
            Expr::UnresolvedMemberAccess(mem_acc) => {
                let obj = match ctx[mem_acc].object {
                    Some(obj) => obj,
                    None => {
                        // member access cannot be resolved yet
                        return Ok(());
                    }
                };

                let obj_type = obj
                    .eval_type(&mut self.symbols, ctx)
                    .map_err(SymbolResolutionError::TypeEval)?;

                let new_expr = match &ctx[&obj_type] {
                    TypeSignatureValue::Struct { name: _ } => {
                        let st_acc = StructAccess {
                            struct_expr: obj,
                            attr_name: *ctx[mem_acc].member_name,
                        }
                        .allocate(ctx);

                        ctx[st_acc].attr_name.parent =
                            IdentParent::StructAccessAttrName(st_acc).into();

                        if let Some(params) = ctx[mem_acc].items.clone() {
                            let st_acc_expr = Expr::StructAccess(st_acc).allocate(ctx);

                            let func_call = FunctionCall {
                                func: st_acc_expr,
                                params,
                            }
                            .allocate(ctx);

                            Expr::FunctionCall(func_call)
                        } else {
                            Expr::StructAccess(st_acc)
                        }
                    }
                    TypeSignatureValue::Enum { name } => {
                        let enm_init = EnumInit {
                            enum_name: *name,
                            enum_value: *ctx[mem_acc].member_name,
                            items: ctx[mem_acc].items.clone().unwrap_or_default(),
                        }
                        .allocate(ctx);

                        ctx[enm_init].enum_name.parent =
                            IdentParent::EnumInitEnumName(enm_init).into();

                        ctx[enm_init].enum_value.parent =
                            IdentParent::EnumInitValueName(enm_init).into();

                        Expr::EnumInit(enm_init)
                    }
                    _ => unreachable!("Expression can never be a member access"),
                };

                ctx[expr] = new_expr;
                // walk the new expression
                walk_expr(self, ctx, scope, expr)?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

pub fn resolve_ident<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    ctx: &mut IrCtx<'a>,
    ident: Ident<'a>,
) -> Result<(), SymbolResolutionError<'a>> {
    let resolved_ident = match &ctx[ident] {
        IdentValue::Unresolved(_) => match *ident.parent {
            IdentParent::StructInitValueName(st_val) => {
                let st_name = *ctx[ctx[st_val].parent].struct_name;

                let st = symbols
                    .lookup(ctx, st_name)
                    .ok_or(SymbolResolutionError::UnknownIdentifier(st_name))?
                    .unwrap_struct(ctx);

                let attr = st
                    .lookup_attr(ident, ctx)
                    .ok_or(SymbolResolutionError::UnknownIdentifier(ident))?;

                Some(*ctx[attr].name)
            }
            IdentParent::StructAccessAttrName(st_access) => {
                let st_attr = st_access
                    .lookup_attr(ctx, symbols)
                    .map_err(|_| SymbolResolutionError::UnknownIdentifier(ident))?;

                match ctx[&*ctx[st_attr].type_sig].clone() {
                    TypeSignatureValue::Unresolved(type_ident) => {
                        let resolved_type_sig = symbols
                            .lookup(ctx, type_ident)
                            .ok_or(SymbolResolutionError::UnknownIdentifier(type_ident))?
                            .clone()
                            .eval_type(symbols, ctx)
                            .map_err(SymbolResolutionError::TypeEval)?;

                        ctx[st_attr].type_sig = resolved_type_sig.into();
                    }
                    _ => {}
                }

                Some(*ctx[st_attr].name)
            }
            IdentParent::EnumInitValueName(enm_init) => {
                let enm_name = ctx[enm_init].enum_name;

                let enm = symbols
                    .lookup(ctx, enm_name)
                    .ok_or(SymbolResolutionError::UnknownIdentifier(enm_name))?
                    .unwrap_enum(ctx);

                let (_, enm_val) = enm.lookup_value(ctx, ctx[enm_init].enum_value).ok_or(
                    SymbolResolutionError::UnknownEnumValue {
                        enm,
                        enum_value: ctx[enm_init].enum_value,
                    },
                )?;

                Some(*ctx[enm_val].name)
            }
            IdentParent::MemberAccessMemberName(_) => None,
            _ => {
                let sym_id = symbols
                    .lookup(ctx, ident)
                    .ok_or(SymbolResolutionError::UnknownIdentifier(ident))?;

                let sym = *&ctx[sym_id];
                Some(sym.name(ctx).into())
            }
        },
        IdentValue::Resolved(_) => None,
    };

    if let Some(sym_ident) = resolved_ident {
        debug_assert_matches!(ctx[sym_ident], IdentValue::Resolved(_));
        ident.parent.change_ident(ctx, sym_ident);
    }

    Ok(())
}
