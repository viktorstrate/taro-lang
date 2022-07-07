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
    fill_type_signature(symbols, elem, None)?;

    let eval_type = elem
        .eval_type(symbols)
        .map_err(TypeCheckerError::TypeEvalError)?;

    // don't allow user to specify type signatures as the Untyped type
    if let Some(type_sig) = elem.specified_type() {
        if *type_sig == BuiltinType::Untyped.type_sig() {
            return Err(TypeCheckerError::UntypedValue(Box::new(elem.clone())));
        }
    }

    if let Some(type_sig) = elem.specified_type() {
        let coerced_type = types_match(type_sig.clone(), eval_type.clone())?;
        elem.specify_type(coerced_type);
    } else {
        // set declaration type to the calculated type of the element
        elem.specify_type(eval_type.clone());
    }

    let type_sig = elem.specified_type().cloned().unwrap_or(eval_type);
    if type_sig == BuiltinType::Untyped.type_sig() {
        return Err(TypeCheckerError::UntypedValue(Box::new(elem.clone())));
    } else if let TypeSignature::Function {
        args: _,
        return_type,
    } = type_sig
    {
        if *return_type == BuiltinType::Untyped.type_sig() {
            return Err(TypeCheckerError::UntypedValue(Box::new(elem.clone())));
        }
    }

    Ok(())
}

pub fn fill_type_signature<'a, T>(
    symbols: &mut SymbolTableZipper<'a>,
    elem: &mut T,
    extra_type_sig: Option<&TypeSignature<'a>>,
) -> Result<(), TypeCheckerError<'a>>
where
    T: 'a + Typed<'a> + Clone,
{
    let specified_type = elem.specified_type().or(extra_type_sig).cloned();

    // If specified type is `Base` then locate the actual type from the symbol table
    let base_ident = match &specified_type {
        Some(TypeSignature::Base(ident)) => Some(ident.clone()),
        None => {
            match elem
                .eval_type(symbols)
                .map_err(TypeCheckerError::TypeEvalError)?
            {
                TypeSignature::Base(ident) => Some(ident),
                _ => None,
            }
        }
        _ => None,
    };

    if let Some(ident) = base_ident {
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
    } else if let Some(type_sig) = extra_type_sig {
        elem.specify_type(type_sig.clone())
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

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ast::test_utils::utils::type_check;
    use crate::parser::parse_ast;

    use super::*;

    #[test]
    fn test_escape_block_var_decl() {
        let mut ast = parse_ast("let a: Number = @{ 1 + 2 }").unwrap();
        assert_matches!(type_check(&mut ast), Ok(_));

        let mut ast = parse_ast("let a = @{ 1 + 2 }").unwrap();
        assert_matches!(type_check(&mut ast), Err(TypeCheckerError::UntypedValue(_)));
    }

    #[test]
    fn test_untyped_function_return() {
        let mut ast = parse_ast("func foo() { return @{ 123 } }").unwrap();
        assert_matches!(type_check(&mut ast), Err(TypeCheckerError::UntypedValue(_)));
    }
}
