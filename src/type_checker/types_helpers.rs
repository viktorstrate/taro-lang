use crate::{
    ir::{
        context::{IrArenaType, IrCtx},
        node::{
            type_signature::{
                BuiltinType, TypeEvalError, TypeSignature, TypeSignatureValue, Typed,
            },
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
    fill_type_signature(ctx, symbols, elem, None)?;
    let specified_type = elem.specified_type(ctx);

    let eval_type = elem
        .eval_type(symbols, ctx)
        .map_err(TypeCheckerError::TypeEvalError)?;

    // don't allow user to specify type signatures as the Untyped type
    if let Some(type_sig) = &specified_type {
        if *type_sig == ctx.get_builtin_type_sig(BuiltinType::Untyped) {
            return Err(TypeCheckerError::UntypedValue());
        }
    }

    if let Some(type_sig) = &specified_type {
        let coerced_type = types_match(ctx, *type_sig, eval_type)?;
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

fn fill_tuple_type_signature<'a, T: IrArenaType<'a>>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    elem: NodeRef<'a, T>,
    specified_type: TypeSignature<'a>,
) -> Result<TypeSignature<'a>, TypeCheckerError<'a>>
where
    NodeRef<'a, T>: Typed<'a>,
{
    // If specified type is `Tuple` then fill types recursiveley instead
    match &ctx[specified_type] {
        TypeSignatureValue::Tuple(type_sigs) => {
            let new_types = type_sigs
                .clone()
                .into_iter()
                .map(|t| fill_tuple_type_signature(ctx, symbols, elem, t))
                .collect::<Result<Vec<_>, _>>()?;

            let new_type = ctx.get_type_sig(TypeSignatureValue::Tuple(new_types));
            elem.specify_type(ctx, new_type)
                .map_err(TypeCheckerError::TypeEvalError)?;

            return Ok(new_type);
        }
        _ => {}
    }

    // If specified type is `Unresolved` then locate the actual type from the symbol table
    let unresolved_ident = match &ctx[specified_type] {
        TypeSignatureValue::Unresolved(ident) => Some(*ident),
        _ => None,
    };

    if let Some(ident) = unresolved_ident {
        let val = *symbols
            .lookup(ctx, ident)
            .ok_or(TypeCheckerError::TypeEvalError(
                TypeEvalError::UnknownIdentifier(ident),
            ))?;

        let new_type = val
            .eval_type(symbols, ctx)
            .map_err(TypeCheckerError::TypeEvalError)?;

        elem.specify_type(ctx, new_type)
            .map_err(TypeCheckerError::TypeEvalError)?;

        Ok(new_type)
    } else {
        Ok(specified_type)
    }
}

pub fn fill_type_signature<'a, T: IrArenaType<'a>>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    elem: NodeRef<'a, T>,
    extra_type_sig: Option<TypeSignature<'a>>,
) -> Result<(), TypeCheckerError<'a>>
where
    NodeRef<'a, T>: Typed<'a>,
{
    let specified_type_id = elem.specified_type(ctx).or(extra_type_sig);

    match specified_type_id {
        Some(type_id) => match &ctx[type_id] {
            TypeSignatureValue::Tuple(_) => {
                fill_tuple_type_signature(ctx, symbols, elem, type_id)?;
            }
            _ => {}
        },
        None => {}
    };

    // If specified type is `Unresolved` then locate the actual type from the symbol table
    let unresolved_ident = match specified_type_id.map(|t| &ctx[t]) {
        Some(TypeSignatureValue::Unresolved(ident)) => Some(*ident),
        None => {
            let type_sig = elem
                .eval_type(symbols, ctx)
                .map_err(TypeCheckerError::TypeEvalError)?;

            match &ctx[type_sig] {
                TypeSignatureValue::Unresolved(ident) => Some(*ident),
                _ => None,
            }
        }
        _ => None,
    };

    if let Some(ident) = unresolved_ident {
        let sym = *symbols
            .lookup(ctx, ident)
            .ok_or(TypeCheckerError::TypeEvalError(
                TypeEvalError::UnknownIdentifier(ident),
            ))?;

        let new_type = sym
            .eval_type(symbols, ctx)
            .map_err(TypeCheckerError::TypeEvalError)?;

        elem.specify_type(ctx, new_type)
            .map_err(TypeCheckerError::TypeEvalError)?;
    } else if let Some(type_sig) = extra_type_sig {
        elem.specify_type(ctx, type_sig)
            .map_err(TypeCheckerError::TypeEvalError)?;
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

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use crate::ir::test_utils::utils::type_check;
//     use crate::parser::parse_ast;

//     use super::*;

//     #[test]
//     fn test_escape_block_var_decl() {
//         let mut ast = parse_ast("let a: Number = @{ 1 + 2 }").unwrap();
//         assert_matches!(type_check(&mut ast), Ok(_));

//         let mut ast = parse_ast("let a = @{ 1 + 2 }").unwrap();
//         assert_matches!(type_check(&mut ast), Err(TypeCheckerError::UntypedValue(_)));
//     }

//     #[test]
//     fn test_untyped_function_return() {
//         let mut ast = parse_ast("func foo() { return @{ 123 } }").unwrap();
//         assert_matches!(type_check(&mut ast), Err(TypeCheckerError::UntypedValue(_)));
//     }
// }
