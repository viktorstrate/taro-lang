use crate::{
    ir::{
        context::IrCtx,
        ir_walker::IrWalker,
        node::{
            enumeration::EnumInit,
            expression::Expr,
            identifier::{Ident, IdentParent, Identifiable},
            type_signature::{TypeEvalError, TypeSignature, TypeSignatureValue},
            IrAlloc, NodeRef,
        },
    },
    symbols::symbol_resolver::SymbolResolutionError,
};

use super::{type_inference::TypeInferrer, TypeChecker, TypeCheckerError};

#[derive(Debug)]
pub struct TypeResolver<'a, 'b>(pub &'b mut TypeChecker<'a>);

impl<'a, 'b> TypeResolver<'a, 'b> {
    pub fn new(
        ctx: &IrCtx<'a>,
        type_inferrer: &'b mut TypeInferrer<'a, '_>,
    ) -> TypeResolver<'a, 'b> {
        type_inferrer.0.symbols.reset(ctx);

        TypeResolver(type_inferrer.0)
    }
}

impl<'a> IrWalker<'a> for TypeResolver<'a, '_> {
    type Error = TypeCheckerError<'a>;

    fn visit_type_sig(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        type_sig: TypeSignature<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        let new_type = self
            .0
            .substitutions
            .get(&type_sig)
            .cloned()
            .unwrap_or(type_sig);

        match ctx[&new_type] {
            TypeSignatureValue::TypeVariable(_) => self.0.found_undeterminable_types = true,
            _ => {}
        }

        Ok(new_type)
    }

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), Self::Error> {
        let mem_acc = match &ctx[expr] {
            Expr::UnresolvedMemberAccess(mem_acc) => *mem_acc,
            _ => return Ok(()),
        };

        // Make sure type sig is resolved before proceeding
        ctx[mem_acc].type_sig = self
            .visit_type_sig(ctx, &mut (), (*ctx[mem_acc].type_sig).clone())?
            .into();

        let enm_init = match &ctx[&*ctx[mem_acc].type_sig] {
            TypeSignatureValue::Enum { name } => {
                let items = match ctx[mem_acc].items.clone() {
                    Some((items, span)) => (Some(items), Some(span)),
                    None => (None, None),
                };

                EnumInit {
                    enum_name: *name,
                    enum_value: *ctx[mem_acc].member_name,
                    items: items.0.unwrap_or_default(),
                    items_span: items.1,
                    span: ctx[mem_acc].span.clone(),
                }
                .allocate(ctx)
            }
            TypeSignatureValue::TypeVariable(_) => {
                self.0.found_undeterminable_types = true;
                return Ok(());
            }
            _ => {
                return Err(TypeCheckerError::AnonymousEnumInitNonEnum(
                    mem_acc,
                    (*ctx[mem_acc].type_sig).clone(),
                ))
            }
        };

        let enm_expr = Expr::EnumInit(enm_init);

        ctx[expr] = enm_expr;

        // Symbol resolve enum name and value
        let enm_name = ctx[enm_init].enum_name;
        let enm = self
            .0
            .symbols
            .lookup(ctx, enm_name)
            .ok_or(TypeCheckerError::SymbolResolutionError(
                SymbolResolutionError::TypeEval(TypeEvalError::UnknownIdent(enm_name)),
            ))?
            .unwrap_enum(ctx);
        let (_, enm_val) = enm.lookup_value(ctx, ctx[enm_init].enum_value).ok_or(
            TypeCheckerError::SymbolResolutionError(SymbolResolutionError::UnknownEnumValue {
                enm,
                enum_value: ctx[enm_init].enum_value,
            }),
        )?;
        ctx[enm_init].enum_value = *ctx[enm_val].name;

        let sym_id =
            self.0
                .symbols
                .lookup(ctx, enm_name)
                .ok_or(TypeCheckerError::SymbolResolutionError(
                    SymbolResolutionError::TypeEval(TypeEvalError::UnknownIdent(enm_name)),
                ))?;

        ctx[enm_init].enum_name = Ident {
            id: (*&ctx[sym_id]).name(ctx).id,
            parent: IdentParent::EnumInitEnumName(enm_init).into(),
        };

        // New types can now potentially be inferred
        self.0.needs_rerun = true;

        Ok(())
    }
}
