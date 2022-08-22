use std::collections::HashMap;

use crate::{
    ir::{
        context::IrCtx,
        ir_walker::IrWalker,
        node::type_signature::{TypeSignature, TypeSignatureValue},
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{type_inference::TypeInferrer, TypeCheckerError};

#[derive(Debug)]
pub struct TypeResolver<'a> {
    pub symbols: SymbolTableZipper<'a>,
    pub substitutions: HashMap<TypeSignature<'a>, TypeSignature<'a>>,
}

impl<'a> TypeResolver<'a> {
    pub fn new(ctx: &IrCtx<'a>, type_inferrer: TypeInferrer<'a>) -> TypeResolver<'a> {
        debug_assert!(type_inferrer.constraints.is_empty());

        let mut symbols = type_inferrer.symbols;
        symbols.reset(ctx);

        TypeResolver {
            symbols,
            substitutions: type_inferrer.substitutions,
        }
    }
}

impl<'a> IrWalker<'a> for TypeResolver<'a> {
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
            TypeSignatureValue::TypeVariable(_) => Err(TypeCheckerError::UndeterminableTypes),
            _ => Ok(new_type),
        }
    }
}
