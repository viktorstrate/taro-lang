use crate::{
    ast::node::{identifier::Ident, structure::StructInit},
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::TypeCheckerError;

#[derive(Debug)]
pub enum StructTypeError<'a> {
    MissingAttribute(Ident<'a>),
    UnknownAttribute(Ident<'a>),
}

pub fn check_struct_init<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    st_init: &StructInit<'a>,
) -> Result<(), TypeCheckerError<'a>> {
    let st = st_init
        .lookup_struct(symbols)
        .ok_or(TypeCheckerError::LookupError(st_init.name.clone()))?;

    // Check that all attributes without default values are declared
    for attr in &st.attrs {
        if attr.default_value.is_none() {
            if st_init
                .values
                .iter()
                .find(|val| val.name == attr.name)
                .is_none()
            {
                return Err(TypeCheckerError::StructError(
                    StructTypeError::MissingAttribute(attr.name.clone()),
                ));
            }
        }
    }

    // Check that declared attributes all exist on struct
    for attr in &st_init.values {
        if st.attrs.iter().find(|val| val.name == attr.name).is_none() {
            return Err(TypeCheckerError::StructError(
                StructTypeError::UnknownAttribute(attr.name.clone()),
            ));
        }
    }

    Ok(())
}
