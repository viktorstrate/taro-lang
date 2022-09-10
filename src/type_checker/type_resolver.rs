use std::collections::HashMap;

use crate::{
    ir::{
        context::IrCtx,
        ir_walker::IrWalker,
        node::{
            enumeration::EnumInit,
            expression::Expr,
            identifier::{Ident, IdentParent, Identifiable},
            type_signature::{TypeSignature, TypeSignatureValue},
            IrAlloc, NodeRef,
        },
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{type_inference::TypeInferrer, TypeCheckerError};

#[derive(Debug)]
pub struct TypeResolver<'a, 'b> {
    pub symbols: &'b mut SymbolTableZipper<'a>,
    pub substitutions: &'b mut HashMap<TypeSignature<'a>, TypeSignature<'a>>,
}

impl<'a, 'b> TypeResolver<'a, 'b> {
    pub fn new(
        ctx: &IrCtx<'a>,
        type_inferrer: &'b mut TypeInferrer<'a, '_>,
    ) -> TypeResolver<'a, 'b> {
        debug_assert!(type_inferrer.0.constraints.is_empty());

        type_inferrer.0.symbols.reset(ctx);

        TypeResolver {
            symbols: &mut type_inferrer.0.symbols,
            substitutions: &mut type_inferrer.0.substitutions,
        }
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
            .substitutions
            .get(&type_sig)
            .cloned()
            .unwrap_or(type_sig);

        match ctx[new_type] {
            TypeSignatureValue::TypeVariable(_) => dbg!(Err(TypeCheckerError::UndeterminableTypes)),
            _ => Ok(new_type),
        }
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
        ctx[mem_acc].type_sig = self.visit_type_sig(ctx, &mut (), ctx[mem_acc].type_sig)?;

        let enm_init = match &ctx[ctx[mem_acc].type_sig] {
            TypeSignatureValue::Enum { name } => EnumInit {
                enum_name: *name,
                enum_value: *ctx[mem_acc].member_name,
                items: ctx[mem_acc].items.clone(),
            }
            .allocate(ctx),
            TypeSignatureValue::TypeVariable(var) => {
                println!("COULD NOT DETERMINE UNRESOLVED MEM ACC {}", var.index());
                return dbg!(Err(TypeCheckerError::UndeterminableTypes));
            }
            _ => unreachable!("Only enums inits can have anonymous base"),
        };

        let enm_expr = Expr::EnumInit(enm_init);

        ctx[expr] = enm_expr;

        // Symbol resolve enum name and value
        let enm_name = ctx[enm_init].enum_name;
        let enm = self
            .symbols
            .lookup(ctx, enm_name)
            .ok_or(TypeCheckerError::LookupError(enm_name))?
            .unwrap_enum(ctx);
        let (_, enm_val) = enm.lookup_value(ctx, ctx[enm_init].enum_value).ok_or(
            TypeCheckerError::UnknownEnumValue {
                enum_name: enm_name,
                enum_value: ctx[enm_init].enum_value,
            },
        )?;
        ctx[enm_init].enum_value = *ctx[enm_val].name;

        let sym_id = self
            .symbols
            .lookup(ctx, enm_name)
            .ok_or(TypeCheckerError::LookupError(enm_name))?;

        ctx[enm_init].enum_name = Ident {
            id: (*&ctx[sym_id]).name(ctx).id,
            parent: IdentParent::EnumInitEnumName(enm_init).into(),
        };

        Ok(())
    }
}
