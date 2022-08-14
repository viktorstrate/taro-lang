use crate::{
    ir::{
        context::{IrArenaType, IrCtx},
        node::{
            type_signature::{BuiltinType, TypeSignature, TypeSignatureValue, Typed},
            NodeRef,
        },
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{coercion::coerce, TypeCheckerError};

pub fn type_check<'a, T: IrArenaType<'a>>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    elem: NodeRef<'a, T>,
) -> Result<(), TypeCheckerError<'a>>
where
    NodeRef<'a, T>: Typed<'a>,
{
    // fill_type_signature(ctx, symbols, elem, None)?;
    let specified_type = elem.specified_type(ctx);

    let eval_type = elem
        .eval_type(symbols, ctx)
        .map_err(TypeCheckerError::TypeEvalError)?;

    // don't allow user to specify type signatures as the Untyped type
    if let Some(type_sig) = specified_type {
        if type_sig == ctx.get_builtin_type_sig(BuiltinType::Untyped) {
            return Err(TypeCheckerError::UntypedValue());
        }
    }

    if let Some(type_sig) = specified_type {
        let coerced_type = types_match(ctx, type_sig, eval_type)?;
        elem.specify_type(ctx, coerced_type)
            .map_err(TypeCheckerError::TypeEvalError)?;
    } else {
        // set declaration type to the calculated type of the element
        elem.specify_type(ctx, eval_type)
            .map_err(TypeCheckerError::TypeEvalError)?;
    }

    let type_sig = specified_type.unwrap_or(eval_type);
    if type_sig == ctx.get_builtin_type_sig(BuiltinType::Untyped) {
        return Err(TypeCheckerError::UntypedValue());
    } else if let TypeSignatureValue::Function {
        args: _,
        return_type,
    } = &ctx[type_sig]
    {
        if *return_type == ctx.get_builtin_type_sig(BuiltinType::Untyped) {
            return Err(TypeCheckerError::UntypedValue());
        }
    }

    Ok(())
}

/// checks that the specified type matches the type of the expression
pub fn types_match<'a>(
    ctx: &IrCtx<'a>,
    type_sig: TypeSignature<'a>,
    expr_type: TypeSignature<'a>,
) -> Result<TypeSignature<'a>, TypeCheckerError<'a>> {
    if let Some(coerced_type) = coerce(type_sig, expr_type, ctx) {
        Ok(coerced_type)
    } else {
        Err(TypeCheckerError::TypeSignatureMismatch::<'a> {
            type_sig,
            expr_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::test_utils::utils::{lowered_ir, type_check};

    use super::*;

    #[test]
    fn test_escape_block_var_decl() {
        let mut ir = lowered_ir("let a: Number = @{ 1 + 2 }").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_));

        let mut ir = lowered_ir("let a = @{ 1 + 2 }").unwrap();
        assert_matches!(type_check(&mut ir), Err(TypeCheckerError::UntypedValue()));
    }

    #[test]
    fn test_untyped_function_return() {
        let mut ir = lowered_ir("func foo() { return @{ 123 } }").unwrap();
        assert_matches!(type_check(&mut ir), Err(TypeCheckerError::UntypedValue()));
    }
}
