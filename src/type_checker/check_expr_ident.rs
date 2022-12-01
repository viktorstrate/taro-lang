use crate::{
    ir::{
        context::IrCtx,
        node::{expression::Expr, identifier::Ident, type_signature::TypeEvalError, NodeRef},
    },
    symbols::symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolValueItem},
};

use super::TypeCheckerError;

pub fn check_expr_ident<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    expr: NodeRef<'a, Expr<'a>>,
    ident: Ident<'a>,
) -> Result<(), TypeCheckerError<'a>> {
    let sym_val = symbols
        .lookup(ctx, ident)
        .ok_or(TypeCheckerError::TypeEval(TypeEvalError::UnknownIdent(
            ident,
        )))?;

    let non_expr = match ctx[sym_val] {
        SymbolValueItem::BuiltinType(_) => true,
        SymbolValueItem::StructDecl(_) => true,
        _ => false,
    };

    if non_expr {
        Err(TypeCheckerError::IdentNotExpression(expr, sym_val))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ir::test_utils::utils::{lowered_ir, type_check},
        type_checker::TypeCheckerError,
    };

    #[test]
    fn test_non_ident_expr_builtin() {
        let mut ir = lowered_ir("String").unwrap();
        assert_matches!(
            type_check(&mut ir).1,
            Err(TypeCheckerError::IdentNotExpression(_, _))
        )
    }

    #[test]
    fn test_non_ident_expr_struct() {
        let mut ir = lowered_ir("struct Foo {}; let x = Foo").unwrap();
        assert_matches!(
            type_check(&mut ir).1,
            Err(TypeCheckerError::IdentNotExpression(_, _))
        )
    }
}
