use crate::{
    ir::{
        context::IrCtx,
        node::{
            assignment::Assignment,
            expression::Expr,
            identifier::Ident,
            type_signature::{Mutability, TypeEvalError},
            NodeRef,
        },
    },
    symbols::{
        symbol_resolver::SymbolResolutionError,
        symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolValueItem},
    },
};

use super::TypeCheckerError;

#[derive(Debug)]
pub enum AssignmentError<'a> {
    ImmutableAssignment(Ident<'a>),
    NotLValue(NodeRef<'a, Expr<'a>>),
}

fn check_assignment_expr<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    asg: NodeRef<'a, Assignment<'a>>,
    expr: NodeRef<'a, Expr<'a>>,
) -> Result<(), TypeCheckerError<'a>> {
    match ctx[expr].clone() {
        Expr::Identifier(ident, _) => {
            let sym =
                symbols
                    .lookup(ctx, *ident)
                    .ok_or(TypeCheckerError::SymbolResolutionError(
                        SymbolResolutionError::TypeEval(TypeEvalError::UnknownIdent(*ident)),
                    ))?;

            match &ctx[sym] {
                SymbolValueItem::VarDecl(var_decl) => {
                    if ctx[*var_decl].mutability == Mutability::Immutable {
                        return Err(TypeCheckerError::AssignmentError(
                            asg,
                            AssignmentError::ImmutableAssignment(*ident),
                        ));
                    }
                }
                _ => {
                    return Err(TypeCheckerError::AssignmentError(
                        asg,
                        AssignmentError::NotLValue(ctx[asg].lhs),
                    ));
                }
            }
        }
        Expr::StructAccess(st_access) => {
            let attr = st_access
                .lookup_attr(ctx, symbols)
                .map_err(TypeCheckerError::TypeEval)?;

            if ctx[attr].mutability == Mutability::Immutable {
                return Err(TypeCheckerError::AssignmentError(
                    asg,
                    AssignmentError::ImmutableAssignment(ctx[st_access].attr_name),
                ));
            }

            check_assignment_expr(ctx, symbols, asg, ctx[st_access].struct_expr)?;
        }
        _ => {
            return Err(TypeCheckerError::AssignmentError(
                asg,
                AssignmentError::NotLValue(ctx[asg].lhs),
            ));
        }
    }

    return Ok(());
}

pub fn check_assignment<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    asg: NodeRef<'a, Assignment<'a>>,
) -> Result<(), TypeCheckerError<'a>> {
    // only assign to:
    // - variable
    // - (nested) struct attribute
    // with properties: mutable, same type

    check_assignment_expr(ctx, symbols, asg, ctx[asg].lhs)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::test_utils::utils::{lowered_ir, type_check};

    use super::*;

    #[test]
    fn test_assign_variable() {
        let mut ir = lowered_ir("var foo = 1; foo = 2").unwrap();
        assert_matches!(type_check(&mut ir).1, Ok(_))
    }

    #[test]
    fn test_assign_variable_immutable() {
        let mut ir = lowered_ir("let foo = 1; foo = 2").unwrap();
        assert_matches!(
            type_check(&mut ir).1,
            Err(TypeCheckerError::AssignmentError(
                _,
                AssignmentError::ImmutableAssignment(_)
            ))
        );
    }

    #[test]
    fn test_assign_struct() {
        let mut ir = lowered_ir(
            "struct Foo { var attr: Number }
            var foo = Foo { attr: 1 }
            foo.attr = 2",
        )
        .unwrap();

        assert_matches!(type_check(&mut ir).1, Ok(_));
    }

    #[test]
    fn test_assign_struct_immutable_attr() {
        let mut ir = lowered_ir(
            "struct Foo { let attr: Number }
            var foo = Foo { attr: 1 }
            foo.attr = 2",
        )
        .unwrap();

        assert_matches!(type_check(&mut ir).1, Err(_));
    }

    #[test]
    fn test_assign_struct_immutable() {
        let mut ir = lowered_ir(
            "struct Foo { var attr: Number }
            let foo = Foo { attr: 1 }
            foo.attr = 2",
        )
        .unwrap();

        assert_matches!(type_check(&mut ir).1, Err(_));
    }

    #[test]
    fn test_nested_struct_immutable() {
        let mut ir = lowered_ir(
            "
        struct Deep {
            var inner = false
        }

        struct Foo {
            let bar: Deep
        }

        let foo = Foo { bar: Deep {} }
        foo.bar.inner = true
        ",
        )
        .unwrap();
        assert_matches!(type_check(&mut ir).1, Err(_))
    }
}
