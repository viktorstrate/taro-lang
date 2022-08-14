use crate::{
    ir::{
        context::IrCtx,
        node::{
            identifier::{Ident, IdentKey},
            structure::StructInit,
            type_signature::Typed,
            NodeRef,
        },
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{types_helpers::types_match, TypeCheckerError};

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

    // Type check attributes
    symbols
        .enter_scope(ctx, ctx[st_init].scope_name)
        .expect("struct init scope should exist");
    for id in ctx[st_init].values.clone() {
        let attr_name = ctx[id].name;
        let attr_value = ctx[id].value;

        let attr_type = attr_value
            .eval_type(symbols, ctx)
            .map_err(TypeCheckerError::TypeEvalError)?;

        let st_attr_type = ctx[st]
            .attrs
            .clone()
            .into_iter()
            .find(|val| IdentKey::idents_eq(ctx, ctx[*val].name, attr_name))
            .expect("checked earlier")
            .eval_type(symbols, ctx)
            .map_err(TypeCheckerError::TypeEvalError)?;

        let coerced_type = types_match(ctx, st_attr_type, attr_type)?;
        attr_value
            .specify_type(ctx, coerced_type)
            .map_err(TypeCheckerError::TypeEvalError)?;
    }
    symbols.exit_scope(ctx).unwrap();

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use crate::{
//         ir::test_utils::utils::type_check, parser::parse_ast, type_checker::TypeCheckerError,
//     };

//     #[test]
//     fn test_func_decl_inside_struct() {
//         let mut ast = parse_ast(
//             "struct Foo { let attr: () -> Number }
//             let a = Foo { attr: () { return false } }",
//         )
//         .unwrap();
//         assert_matches!(
//             type_check(&mut ast),
//             Err(TypeCheckerError::TypeSignatureMismatch {
//                 type_sig: _,
//                 expr_type: _
//             })
//         )
//     }
// }
