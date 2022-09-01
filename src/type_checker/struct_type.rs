use crate::{
    ir::{
        context::IrCtx,
        node::{
            identifier::{Ident, IdentKey},
            structure::StructInit,
            NodeRef,
        },
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::TypeCheckerError;

#[derive(Debug)]
pub enum StructTypeError<'a> {
    MissingAttribute(Ident<'a>),
    UnknownAttribute(Ident<'a>),
}

pub fn check_struct_init<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    st_init: NodeRef<'a, StructInit<'a>>,
) -> Result<(), TypeCheckerError<'a>> {
    let st_name = ctx[st_init].struct_name;
    let st = st_init
        .lookup_struct(ctx, symbols)
        .ok_or(TypeCheckerError::LookupError(st_name))?;

    // Check that all attributes without default values are declared
    for attr in ctx[st].attrs.clone() {
        if ctx[attr].default_value.is_none() {
            let attr_name = ctx[attr].name;
            if ctx[st_init]
                .values
                .iter()
                .find(|val| IdentKey::idents_eq(ctx, ctx[**val].name, attr_name))
                .is_none()
            {
                return Err(TypeCheckerError::StructError(
                    StructTypeError::MissingAttribute(attr_name),
                ));
            }
        }
    }

    // Check that declared attributes all exist on struct
    for attr in ctx[st_init].values.clone() {
        let attr_name = ctx[attr].name;
        if ctx[st]
            .attrs
            .iter()
            .find(|val| IdentKey::idents_eq(ctx, ctx[**val].name, attr_name))
            .is_none()
        {
            return Err(TypeCheckerError::StructError(
                StructTypeError::UnknownAttribute(attr_name),
            ));
        }
    }

    Ok(())
}
