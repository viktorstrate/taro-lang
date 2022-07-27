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

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::test_utils::utils::type_check, parser::parse_ast, type_checker::TypeCheckerError,
    };

    #[test]
    fn test_func_decl_inside_struct() {
        let mut ast = parse_ast(
            "struct Foo { let attr: () -> Number }
            let a = Foo { attr: () { return false } }",
        )
        .unwrap();
        assert_matches!(
            type_check(&mut ast),
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig: _,
                expr_type: _
            })
        )
    }
}
