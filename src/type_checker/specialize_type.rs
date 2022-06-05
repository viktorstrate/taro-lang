use crate::{
    ast::node::{
        expression::ExprValueError,
        type_signature::{TypeSignature, Typed},
    },
    symbols::symbol_table_zipper::SymbolTableZipper,
};

use super::TypeCheckerError;

/// If `base_type` is `TypeSignature::Base`, then look for full type signature in the symbols table.
pub fn specialize_type<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    base_type: &mut TypeSignature<'a>,
) -> Result<(), TypeCheckerError<'a>> {
    match base_type {
        TypeSignature::Base(ident) => {
            let val = symbols
                .locate(&ident)
                .ok_or(TypeCheckerError::ValueError(
                    ExprValueError::UnknownIdentifier(ident.clone()),
                ))?
                .clone();

            *base_type = val
                .type_sig(symbols)
                .map_err(TypeCheckerError::ValueError)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
