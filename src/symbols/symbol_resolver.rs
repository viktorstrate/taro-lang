use std::assert_matches::debug_assert_matches;

use crate::{
    ir::{
        context::IrCtx,
        ir_walker::{IrWalkable, IrWalker, ScopeValue},
        node::{
            enumeration::{Enum, EnumInit},
            expression::Expr,
            function::FunctionCall,
            identifier::{Ident, IdentParent, IdentValue, Identifiable},
            member_access::UnresolvedMemberAccess,
            statement::{Stmt, VarDecl},
            structure::StructAccess,
            type_signature::{TypeEvalError, TypeSignature, TypeSignatureValue, Typed},
            IrAlloc, NodeRef,
        },
    },
    parser::Span,
};

use super::symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolTable};

#[derive(Debug)]
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
    TypeEval(TypeEvalError<'a>),
    UnknownEnumValue {
        enm: NodeRef<'a, Enum<'a>>,
        enum_value: Ident<'a>,
    },
    InvalidMemberAccessType {
        mem_acc: NodeRef<'a, UnresolvedMemberAccess<'a>>,
        obj_type: TypeSignature<'a>,
    },
    RecursiveDeclaration {
        var_decl: NodeRef<'a, VarDecl<'a>>,
        ident_span: Span<'a>,
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
                let sym_val =
                    self.symbols
                        .lookup(ctx, ident)
                        .ok_or(SymbolResolutionError::TypeEval(
                            TypeEvalError::UnknownIdent(ident),
                        ))?;

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

    fn visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        stmt: NodeRef<'a, Stmt<'a>>,
    ) -> Result<(), Self::Error> {
        match ctx[stmt] {
            Stmt::VariableDecl(var_decl) => {
                let mut ident_searcher = SearchIdentWalker {
                    search_ident: *ctx[var_decl].name,
                };

                ctx[var_decl]
                    .value
                    .walk(&mut ident_searcher, ctx, scope)
                    .map_err(|span| SymbolResolutionError::RecursiveDeclaration {
                        var_decl: var_decl,
                        ident_span: span,
                    })?;
            }
            _ => {}
        }

        Ok(())
    }

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), Self::Error> {
        match &ctx[expr] {
            Expr::UnresolvedMemberAccess(mem_acc) => {
                let mem_acc = *mem_acc;
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

                        if let Some((args, args_span)) = ctx[mem_acc].items.clone() {
                            let st_acc_expr = Expr::StructAccess(st_acc).allocate(ctx);

                            let func_call = FunctionCall {
                                func: st_acc_expr,
                                args,
                                args_span,
                            }
                            .allocate(ctx);

                            Expr::FunctionCall(func_call)
                        } else {
                            Expr::StructAccess(st_acc)
                        }
                    }
                    TypeSignatureValue::Enum { name } => {
                        let items = match ctx[mem_acc].items.clone() {
                            Some((items, span)) => (Some(items), Some(span)),
                            None => (None, None),
                        };

                        let enm_init = EnumInit {
                            enum_name: *name,
                            enum_value: *ctx[mem_acc].member_name,
                            items: items.0.unwrap_or_default(),
                            items_span: items.1,
                            span: ctx[mem_acc].span.clone(),
                        }
                        .allocate(ctx);

                        ctx[enm_init].enum_name.parent =
                            IdentParent::EnumInitEnumName(enm_init).into();

                        ctx[enm_init].enum_value.parent =
                            IdentParent::EnumInitValueName(enm_init).into();

                        Expr::EnumInit(enm_init)
                    }
                    _ => {
                        return Err(SymbolResolutionError::InvalidMemberAccessType {
                            mem_acc,
                            obj_type,
                        });
                    }
                };

                ctx[expr] = new_expr;
                // walk the new expression
                expr.walk(self, ctx, scope)?;

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
                if let Some(st_name) = ctx[st_val].parent.struct_name(ctx) {
                    let st = symbols
                        .lookup(ctx, st_name)
                        .ok_or(SymbolResolutionError::TypeEval(
                            TypeEvalError::UnknownIdent(st_name),
                        ))?
                        .unwrap_struct(ctx);

                    let attr =
                        st.lookup_attr(ident, ctx)
                            .ok_or(SymbolResolutionError::TypeEval(
                                TypeEvalError::UnknownIdent(ident),
                            ))?;

                    Some(*ctx[attr].name)
                } else {
                    None
                }
            }
            IdentParent::StructAccessAttrName(st_access) => {
                let st_attr = st_access.lookup_attr(ctx, symbols).map_err(|_| {
                    SymbolResolutionError::TypeEval(TypeEvalError::UnknownIdent(ident))
                })?;

                match ctx[&*ctx[st_attr].type_sig].clone() {
                    TypeSignatureValue::Unresolved(type_ident) => {
                        let resolved_type_sig = symbols
                            .lookup(ctx, type_ident)
                            .ok_or(SymbolResolutionError::TypeEval(
                                TypeEvalError::UnknownIdent(type_ident),
                            ))?
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
                    .ok_or(SymbolResolutionError::TypeEval(
                        TypeEvalError::UnknownIdent(enm_name),
                    ))?
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
                    .ok_or(SymbolResolutionError::TypeEval(
                        TypeEvalError::UnknownIdent(ident),
                    ))?;

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

/// Walker used to search a subtree for a given identifier expression.
/// If a match is found the walker returns an "error" with the span of the found identifier.
struct SearchIdentWalker<'a> {
    pub search_ident: Ident<'a>,
}

impl<'a> IrWalker<'a> for SearchIdentWalker<'a> {
    type Error = Span<'a>;

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), Self::Error> {
        match &ctx[expr] {
            Expr::Identifier(id, span) => {
                if **id == self.search_ident {
                    return Err(span.clone());
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::test_utils::utils::{lowered_ir, resolve_symbols};

    use super::SymbolResolutionError;

    #[test]
    fn test_self_referencing_var_decl() {
        let mut ir = lowered_ir("let x = x").unwrap();
        assert_matches!(
            resolve_symbols(&mut ir),
            Err(SymbolResolutionError::RecursiveDeclaration {
                var_decl: _,
                ident_span: _
            })
        )
    }

    #[test]
    fn test_indirect_self_referencing_var_decl() {
        let mut ir = lowered_ir("let x = (123, x)").unwrap();
        assert_matches!(
            resolve_symbols(&mut ir),
            Err(SymbolResolutionError::RecursiveDeclaration {
                var_decl: _,
                ident_span: _
            })
        )
    }
}
