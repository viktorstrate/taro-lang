use crate::{
    ir::{
        context::IrCtx,
        node::{
            identifier::{Ident, IdentKey},
            structure::StructInit,
            type_signature::TypeEvalError,
            NodeRef,
        },
    },
    symbols::{
        symbol_resolver::SymbolResolutionError,
        symbol_table::symbol_table_zipper::SymbolTableZipper,
    },
};

use super::TypeCheckerError;

#[derive(Debug)]
pub enum StructTypeError<'a> {
    MissingAttribute(NodeRef<'a, StructInit<'a>>, Ident<'a>),
    UnknownAttribute(Ident<'a>),
}

pub fn check_struct_init<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    st_init: NodeRef<'a, StructInit<'a>>,
) -> Result<(), TypeCheckerError<'a>> {
    let st_name = st_init.struct_name(ctx).expect("should be resolved by now");
    // .struct_name
    // .expect("struct_name should have been resolved by now");
    let st = st_init
        .lookup_struct(ctx, symbols)
        .ok_or(TypeCheckerError::SymbolResolutionError(
            SymbolResolutionError::TypeEval(TypeEvalError::UnknownIdent(st_name)),
        ))?;

    // Check that all attributes without default values are declared
    for attr in ctx[st].attrs.clone() {
        if ctx[attr].default_value.is_none() {
            let attr_name = *ctx[attr].name;
            if ctx[st_init]
                .values
                .iter()
                .find(|val| IdentKey::idents_eq(ctx, *ctx[**val].name, attr_name))
                .is_none()
            {
                return Err(TypeCheckerError::StructError(
                    st,
                    StructTypeError::MissingAttribute(st_init, attr_name),
                ));
            }
        }
    }

    // Check that declared attributes all exist on struct
    for attr in ctx[st_init].values.clone() {
        let attr_name = *ctx[attr].name;
        if ctx[st]
            .attrs
            .iter()
            .find(|val| IdentKey::idents_eq(ctx, *ctx[**val].name, attr_name))
            .is_none()
        {
            return Err(TypeCheckerError::StructError(
                st,
                StructTypeError::UnknownAttribute(attr_name),
            ));
        }
    }

    Ok(())
}
