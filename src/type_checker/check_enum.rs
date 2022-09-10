use crate::{
    ir::{
        context::IrCtx,
        node::{enumeration::EnumInit, NodeRef},
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::TypeCheckerError;

pub fn check_enum_init<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    enm_init: NodeRef<'a, EnumInit<'a>>,
) -> Result<(), TypeCheckerError<'a>> {
    let enm_name = ctx[enm_init].enum_name;
    let enm = enm_init
        .lookup_enum(ctx, symbols)
        .ok_or(TypeCheckerError::LookupError(enm_name))?;

    let enm_val = enm
        .lookup_value(ctx, ctx[enm_init].enum_value)
        .ok_or(TypeCheckerError::LookupError(enm_name))?
        .1;

    if ctx[enm_val].items.len() != ctx[enm_init].items.len() {
        return Err(TypeCheckerError::EnumInitArgCountMismatch(
            enm_init, enm_val,
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::test_utils::utils::{lowered_ir, type_check};

    use super::*;

    #[test]
    fn test_enum_init_arg_count_mismatch() {
        let mut ir = lowered_ir(
            "
        enum Foo { bar(Number, Number) }\n\
        let x: Foo = .bar(1, 2, 3)
        ",
        )
        .unwrap();

        assert_matches!(
            type_check(&mut ir),
            Err(TypeCheckerError::EnumInitArgCountMismatch(_, _))
        );
    }

    #[test]
    fn test_nested_implicit_enum() {
        let mut ir = lowered_ir(
            "
        enum A { inner(Number) }\n\
        enum B { outer(A) }\n\
        let x: B = .outer(.inner(42))
        ",
        )
        .unwrap();

        assert_matches!(type_check(&mut ir), Ok(_));
    }
}
