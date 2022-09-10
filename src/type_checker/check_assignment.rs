use crate::{
    ir::{
        context::IrCtx,
        node::{
            assignment::Assignment,
            expression::Expr,
            identifier::Ident,
            type_signature::{Mutability, TypeSignature},
            NodeRef,
        },
    },
    symbols::symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolValueItem},
};

use super::TypeCheckerError;

#[derive(Debug)]
pub enum AssignmentError<'a> {
    ImmutableAssignment(Ident<'a>),
    NotLValue(NodeRef<'a, Expr<'a>>),
    TypesMismatch {
        lhs: TypeSignature<'a>,
        rhs: TypeSignature<'a>,
    },
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

    match ctx[ctx[asg].lhs].clone() {
        Expr::Identifier(ident) => {
            let sym = symbols
                .lookup(ctx, *ident)
                .ok_or(TypeCheckerError::LookupError(*ident))?;

            match &ctx[sym] {
                SymbolValueItem::VarDecl(var_decl) => {
                    if ctx[*var_decl].mutability == Mutability::Immutable {
                        return Err(TypeCheckerError::AssignmentError(
                            AssignmentError::ImmutableAssignment(*ident),
                        ));
                    }
                }
                _ => {
                    return Err(TypeCheckerError::AssignmentError(
                        AssignmentError::NotLValue(ctx[asg].lhs),
                    ));
                }
            }
        }
        Expr::StructAccess(st_access) => {
            let attrs = st_access
                .lookup_attr_chain(ctx, symbols)
                .map_err(TypeCheckerError::TypeEval)?;

            if !attrs
                .iter()
                .all(|a| ctx[*a].mutability == Mutability::Mutable)
            {
                return Err(TypeCheckerError::AssignmentError(
                    AssignmentError::ImmutableAssignment(ctx[st_access].attr_name),
                ));
            }
        }
        _ => {
            return Err(TypeCheckerError::AssignmentError(
                AssignmentError::NotLValue(ctx[asg].lhs),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::test_utils::utils::{lowered_ir, type_check};

    use super::*;

    #[test]
    fn test_assign_variable() {
        let mut ir = lowered_ir("let mut foo = 1; foo = 2").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_))
    }

    #[test]
    fn test_assign_variable_immutable() {
        let mut ir = lowered_ir("let foo = 1; foo = 2").unwrap();
        assert_matches!(
            type_check(&mut ir),
            Err(TypeCheckerError::AssignmentError(
                AssignmentError::ImmutableAssignment(_)
            ))
        );
    }

    #[test]
    fn test_assign_struct() {
        let mut ir = lowered_ir(
            "struct Foo { let mut attr: Number }
            let mut foo = Foo { attr: 1 }
            foo.attr = 2",
        )
        .unwrap();

        assert_matches!(type_check(&mut ir), Ok(_));
    }

    #[test]
    fn test_assign_struct_immutable() {
        let mut ir = lowered_ir(
            "struct Foo { let attr: Number }
            let mut foo = Foo { attr: 1 }
            foo.attr = 2",
        )
        .unwrap();

        assert_matches!(type_check(&mut ir), Err(_));
    }

    #[test]
    fn test_nested_struct_immutable() {
        let mut ir = lowered_ir(
            "
        struct Deep {
            let mut inner = false
        }

        struct Foo {
            let bar: Deep
        }

        let foo = Foo { bar: Deep {} }
        foo.bar.inner = true
        ",
        )
        .unwrap();
        assert_matches!(type_check(&mut ir), Err(_))
    }
}
