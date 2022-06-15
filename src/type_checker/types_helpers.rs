use crate::{
    ast::node::type_signature::{TypeEvalError, TypeSignature, Typed},
    symbols::{builtin_types::BuiltinType, symbol_table::symbol_table_zipper::SymbolTableZipper},
};

use super::TypeCheckerError;

pub fn type_check<'a, T>(
    symbols: &mut SymbolTableZipper<'a>,
    elem: &mut T,
) -> Result<(), TypeCheckerError<'a>>
where
    T: 'a + Typed<'a> + Clone,
{
    fill_type_signature(symbols, elem)?;

    let eval_type = elem
        .eval_type(symbols)
        .map_err(TypeCheckerError::TypeEvalError)?;

    if let Some(type_sig) = elem.specified_type() {
        if *type_sig == BuiltinType::Untyped.type_sig() {
            return Err(TypeCheckerError::UntypedValue(Box::new(elem.clone())));
        }
    }

    if let Some(type_sig) = elem.specified_type() {
        types_match(type_sig.clone(), eval_type)?;
    } else {
        // set declaration type to the calculated type of the element
        elem.specify_type(eval_type);
    }

    if let Some(type_sig) = elem.specified_type() {
        if *type_sig == BuiltinType::Untyped.type_sig() {
            return Err(TypeCheckerError::UntypedValue(Box::new(elem.clone())));
        }
    }

    Ok(())
}

pub fn fill_type_signature<'a, T>(
    symbols: &mut SymbolTableZipper<'a>,
    elem: &mut T,
) -> Result<(), TypeCheckerError<'a>>
where
    T: 'a + Typed<'a> + Clone,
{
    // If specified type is `Base` then locate the actual type from the symbol table
    match elem.specified_type() {
        Some(TypeSignature::Base(ident)) => {
            let val = symbols
                .lookup(&ident)
                .ok_or(TypeCheckerError::TypeEvalError(
                    TypeEvalError::UnknownIdentifier(ident.clone()),
                ))?
                .clone();

            let new_type = val
                .eval_type(symbols)
                .map_err(TypeCheckerError::TypeEvalError)?;

            elem.specify_type(new_type);
        }
        _ => {}
    }

    Ok(())
}

/// checks that the specified type matches the type of the expression
pub fn types_match<'a>(
    type_sig: TypeSignature<'a>,
    expr_type: TypeSignature<'a>,
) -> Result<(), TypeCheckerError<'a>> {
    if expr_type == BuiltinType::Untyped.type_sig() {
        // Allow untyped to be assigned as any type
        return Ok(());
    }

    if expr_type != type_sig {
        return Err(TypeCheckerError::TypeSignatureMismatch::<'a> {
            type_sig,
            expr_type,
        });
    }

    Ok(())
}
