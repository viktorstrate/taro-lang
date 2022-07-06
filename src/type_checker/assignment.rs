use crate::{
    ast::node::{
        assignment::Assignment,
        expression::Expr,
        identifier::Ident,
        type_signature::{Mutability, TypeSignature, Typed},
    },
    symbols::symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolValue},
};

use super::TypeCheckerError;

#[derive(Debug)]
pub enum AssignmentError<'a> {
    ImmutableAssignment(Ident<'a>),
    NotLValue(Expr<'a>),
    TypesMismatch {
        lhs: TypeSignature<'a>,
        rhs: TypeSignature<'a>,
    },
}

pub fn check_assignment<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    asg: &Assignment<'a>,
) -> Result<(), TypeCheckerError<'a>> {
    // only assign to:
    // - variable
    // - (nested) struct attribute
    // with properties: mutable, same type

    match &asg.lhs {
        Expr::Identifier(ident) => {
            let var = symbols
                .lookup(&ident)
                .ok_or(TypeCheckerError::LookupError(ident.clone()))?;

            match var {
                SymbolValue::VarDecl(var_decl) => {
                    if var_decl.mutability == Mutability::Immutable {
                        return Err(TypeCheckerError::AssignmentError(
                            AssignmentError::ImmutableAssignment(ident.clone()),
                        ));
                    }
                }
                _ => {
                    return Err(TypeCheckerError::AssignmentError(
                        AssignmentError::NotLValue(asg.lhs.clone()),
                    ));
                }
            }
        }
        Expr::StructAccess(st_access) => {
            let attrs = st_access
                .lookup_attr_chain(symbols)
                .map_err(TypeCheckerError::TypeEvalError)?;

            if !attrs.iter().all(|a| a.mutability == Mutability::Mutable) {
                return Err(TypeCheckerError::AssignmentError(
                    AssignmentError::ImmutableAssignment(st_access.attr_name.clone()),
                ));
            }
        }
        _ => {
            return Err(TypeCheckerError::AssignmentError(
                AssignmentError::NotLValue(asg.lhs.clone()),
            ));
        }
    }

    let lhs_type = asg
        .lhs
        .eval_type(symbols)
        .map_err(TypeCheckerError::TypeEvalError)?;
    let rhs_type = asg
        .rhs
        .eval_type(symbols)
        .map_err(TypeCheckerError::TypeEvalError)?;

    if !rhs_type.can_coerce_to(&lhs_type) {
        return Err(TypeCheckerError::AssignmentError(
            AssignmentError::TypesMismatch {
                lhs: lhs_type,
                rhs: rhs_type,
            },
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ast::test_utils::utils::type_check;
    use crate::parser::parse_ast;

    use super::*;

    #[test]
    fn test_assign_variable() {
        let mut ast = parse_ast("let mut foo = 1; foo = 2").unwrap();
        assert_matches!(type_check(&mut ast), Ok(()))
    }

    #[test]
    fn test_assign_variable_immutable() {
        let mut ast = parse_ast("let foo = 1; foo = 2").unwrap();
        assert_matches!(
            type_check(&mut ast),
            Err(TypeCheckerError::AssignmentError(
                AssignmentError::ImmutableAssignment(_)
            ))
        );
    }

    #[test]
    fn test_assign_variable_types_mismatch() {
        let mut ast = parse_ast("let mut foo = 1; foo = false").unwrap();
        assert_matches!(
            type_check(&mut ast),
            Err(TypeCheckerError::AssignmentError(
                AssignmentError::TypesMismatch { lhs: _, rhs: _ }
            ))
        );
    }

    #[test]
    fn test_assign_struct() {
        let mut ast = parse_ast(
            "struct Foo { let mut attr: Number }
            let mut foo = Foo { attr: 1 }
            foo.attr = 2",
        )
        .unwrap();

        assert_matches!(type_check(&mut ast), Ok(()));
    }

    #[test]
    fn test_assign_struct_immutable() {
        let mut ast = parse_ast(
            "struct Foo { let attr: Number }
            let mut foo = Foo { attr: 1 }
            foo.attr = 2",
        )
        .unwrap();

        assert_matches!(type_check(&mut ast), Err(_));
    }

    #[test]
    fn test_nested_struct_immutable() {
        let mut ast = parse_ast(
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
        assert_matches!(type_check(&mut ast), Err(_))
    }
}
