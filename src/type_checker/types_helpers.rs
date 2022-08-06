use crate::{
    ir::{
        context::IrCtx,
        node::type_signature::{
            BuiltinType, TypeEvalError, TypeSignature, TypeSignatureValue, Typed,
        },
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::TypeCheckerError;

pub fn type_check<'a, T>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    elem: &mut T,
) -> Result<(), TypeCheckerError<'a>>
where
    T: 'a + Typed<'a> + Clone,
{
    fill_type_signature(ctx, symbols, elem, None)?;
    let specified_type = elem.specified_type();

    let eval_type = elem
        .eval_type(symbols)
        .map_err(TypeCheckerError::TypeEvalError)?;

    // don't allow user to specify type signatures as the Untyped type
    if let Some(type_sig) = &specified_type {
        if *type_sig == ctx.get_builtin_type_sig(BuiltinType::Untyped) {
            return Err(TypeCheckerError::UntypedValue(Box::new(elem.clone())));
        }
    }

    if let Some(type_sig) = &specified_type {
        let coerced_type = types_match(type_sig.clone(), eval_type.clone())?;
        elem.specify_type(coerced_type)
            .map_err(TypeCheckerError::TypeEvalError)?;
    } else {
        // set declaration type to the calculated type of the element
        elem.specify_type(eval_type.clone())
            .map_err(TypeCheckerError::TypeEvalError)?;
    }

    let type_sig = specified_type.unwrap_or(eval_type);
    if type_sig == ctx.get_builtin_type_sig(BuiltinType::Untyped) {
        return Err(TypeCheckerError::UntypedValue(Box::new(elem.clone())));
    } else if let TypeSignatureValue::Function {
        args: _,
        return_type,
    } = &ctx.types[type_sig]
    {
        if *return_type == ctx.get_builtin_type_sig(BuiltinType::Untyped) {
            return Err(TypeCheckerError::UntypedValue(Box::new(elem.clone())));
        }
    }

    Ok(())
}

fn fill_tuple_type_signature<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    specified_type: TypeSignature<'a>,
) -> Result<(), TypeCheckerError<'a>> {
    // If specified type is `Tuple` then fill types recursiveley instead
    if let TypeSignatureValue::Tuple(type_sigs) = &ctx.types[specified_type] {
        for type_sig in type_sigs.clone() {
            fill_tuple_type_signature(ctx, symbols, type_sig)?;
        }
        return Ok(());
    }

    // If specified type is `Unresolved` then locate the actual type from the symbol table
    let unresolved_ident = match &ctx.types[specified_type] {
        TypeSignatureValue::Unresolved(ident) => Some(*ident),
        _ => None,
    };

    if let Some(ident) = unresolved_ident {
        let val_id = *symbols
            .lookup(ctx, ident)
            .ok_or(TypeCheckerError::TypeEvalError(
                TypeEvalError::UnknownIdentifier(ident.clone()),
            ))?;

        let val = &ctx.symbols[val_id];

        let new_type = val
            .eval_type(symbols)
            .map_err(TypeCheckerError::TypeEvalError)?;

        *specified_type = new_type;
    }

    Ok(())
}

pub fn fill_type_signature<'a, T>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    elem: &mut T,
    extra_type_sig: Option<TypeSignature<'a>>,
) -> Result<(), TypeCheckerError<'a>>
where
    T: 'a + Typed<'a> + Clone,
{
    let specified_type_id = elem.specified_type().or(extra_type_sig.clone());

    if let Some(type_sig @ TypeSignature::Tuple(_)) = &ctx.types[specified_type_id] {
        fill_tuple_type_signature(ctx, symbols, type_sig)?;
        elem.specify_type(type_sig.clone())
            .map_err(TypeCheckerError::TypeEvalError)?;
    }

    // If specified type is `Unresolved` then locate the actual type from the symbol table
    let unresolved_ident = match &ctx.types[specified_type_id] {
        Some(TypeSignatureValue::Unresolved(ident)) => Some(*ident),
        None => {
            match elem
                .eval_type(symbols)
                .map_err(TypeCheckerError::TypeEvalError)?
            {
                TypeSignatureValue::Unresolved(ident) => Some(ident),
                _ => None,
            }
        }
        _ => None,
    };

    if let Some(ident) = unresolved_ident {
        let val_id = *symbols
            .lookup(ctx, ident)
            .ok_or(TypeCheckerError::TypeEvalError(
                TypeEvalError::UnknownIdentifier(ident),
            ))?;

        let val = &ctx.symbols[val_id];

        let new_type = val
            .eval_type(symbols)
            .map_err(TypeCheckerError::TypeEvalError)?;

        elem.specify_type(new_type)
            .map_err(TypeCheckerError::TypeEvalError)?;
    } else if let Some(type_sig) = extra_type_sig {
        elem.specify_type(type_sig.clone())
            .map_err(TypeCheckerError::TypeEvalError)?;
    }

    Ok(())
}

/// checks that the specified type matches the type of the expression
pub fn types_match<'a>(
    type_sig: TypeSignature<'a>,
    expr_type: TypeSignature<'a>,
) -> Result<TypeSignature<'a>, TypeCheckerError<'a>> {
    if let Some(coerced_type) = TypeSignature::coerce(&type_sig, &expr_type) {
        Ok(coerced_type.clone())
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
