use std::io::Write;

use crate::{
    ir::{context::IrCtx, node::type_signature::TypeEvalError},
    parser::Span,
    type_checker::{
        check_assignment::AssignmentError, FunctionError, TypeChecker, TypeCheckerError,
    },
};

use super::ErrorMessage;

impl<'a, W: Write> ErrorMessage<'a, (&TypeChecker<'a>, &IrCtx<'a>), W> for TypeCheckerError<'a> {
    fn err_span(&self, ctx: (&TypeChecker<'a>, &IrCtx<'a>)) -> Option<Span<'a>> {
        let (type_checker, ctx) = ctx;

        match self {
            TypeCheckerError::ConflictingTypes(a, b) => a.get_span(ctx).or_else(|| b.get_span(ctx)),
            TypeCheckerError::UndeterminableTypes => {
                let t = type_checker.constraints.front().unwrap().0.clone();
                t.get_span(ctx)
            }
            TypeCheckerError::TypeEval(_) => todo!(),
            TypeCheckerError::LookupError(_) => todo!(),
            TypeCheckerError::AssignmentError(_) => todo!(),
            TypeCheckerError::StructError(_) => todo!(),
            TypeCheckerError::FunctionError(func_err) => match func_err {
                FunctionError::ArgCountMismatch(_, _) => todo!(),
                FunctionError::FuncCallWrongArgAmount(_) => todo!(),
            },
            TypeCheckerError::UnknownEnumValue {
                enum_name: _,
                enum_value: _,
            } => todo!(),
            TypeCheckerError::EnumInitArgCountMismatch(_, _) => todo!(),
        }
    }

    fn err_title(
        &self,
        w: &mut W,
        _ctx: (&TypeChecker<'a>, &IrCtx<'a>),
    ) -> Result<(), std::io::Error> {
        match self {
            TypeCheckerError::ConflictingTypes(_, _) => write!(w, "conflicting types"),
            TypeCheckerError::UndeterminableTypes => write!(w, "undeterminable types"),
            TypeCheckerError::TypeEval(type_eval) => match type_eval {
                TypeEvalError::Expression(_) => todo!(),
                TypeEvalError::CallNonFunction(_) => write!(w, "call non-function"),
                TypeEvalError::FuncWrongNumberOfArgs {
                    func: _,
                    expected: _,
                    actual: _,
                } => todo!(),
                TypeEvalError::AccessNonStruct(_) => write!(w, "access non-struct"),
                TypeEvalError::AccessNonTuple(_) => write!(w, "access non-tuple"),
                TypeEvalError::AccessNonEnum(_) => write!(w, "access non-enum"),
                TypeEvalError::TupleAccessOutOfBounds {
                    tuple_len: _,
                    access_item: _,
                } => todo!(),
                TypeEvalError::UnknownIdent(_) => write!(w, "unknown identifier"),
                TypeEvalError::UndeterminableType(_) => write!(w, "undeterminable types"),
            },
            TypeCheckerError::LookupError(_) => write!(w, "unknown identifier"),
            TypeCheckerError::AssignmentError(asgn_err) => match asgn_err {
                AssignmentError::ImmutableAssignment(_) => write!(w, "immutable assignment"),
                AssignmentError::NotLValue(_) => write!(w, "assignment error"),
                AssignmentError::TypesMismatch { lhs: _, rhs: _ } => write!(w, "conflicting types"),
            },
            TypeCheckerError::StructError(st_err) => match st_err {
                crate::type_checker::check_struct::StructTypeError::MissingAttribute(_) => {
                    write!(w, "struct is missing attribute")
                }
                crate::type_checker::check_struct::StructTypeError::UnknownAttribute(_) => {
                    write!(w, "struct has unknown attribute")
                }
            },
            TypeCheckerError::FunctionError(func_err) => match func_err {
                FunctionError::ArgCountMismatch(_, _) => todo!(),
                FunctionError::FuncCallWrongArgAmount(_) => {
                    write!(w, "call function with wrong number of arguments")
                }
            },
            TypeCheckerError::UnknownEnumValue {
                enum_name: _,
                enum_value: _,
            } => write!(w, "unknown enum value"),
            TypeCheckerError::EnumInitArgCountMismatch(_, _) => {
                write!(w, "enum initialized with wrong number of arguments")
            }
        }
    }

    fn err_msg(
        &self,
        w: &mut W,
        ctx: (&TypeChecker<'a>, &IrCtx<'a>),
    ) -> Result<(), std::io::Error> {
        let (_type_checker, ctx) = ctx;

        match self {
            TypeCheckerError::ConflictingTypes(a, b) => {
                writeln!(w, "the following types are not equatable:\n")?;

                if let Some(span) = b.get_span(ctx) {
                    span.format_spanned_code(
                        w,
                        Some(format!("of type '{}'\n", b.format(ctx)).as_str()),
                    )?;
                } else {
                    writeln!(w, "  - type '{}'\n", b.format(ctx).as_str())?;
                }

                if let Some(span) = a.get_span(ctx) {
                    span.format_spanned_code(
                        w,
                        Some(format!("of type '{}'", a.format(ctx)).as_str()),
                    )?;
                } else {
                    writeln!(w, "  - type '{}'\n", a.format(ctx).as_str())?;
                }
            }
            TypeCheckerError::UndeterminableTypes => todo!(),
            TypeCheckerError::TypeEval(_) => todo!(),
            TypeCheckerError::LookupError(_) => todo!(),
            TypeCheckerError::AssignmentError(_) => todo!(),
            TypeCheckerError::StructError(_) => todo!(),
            TypeCheckerError::FunctionError(_) => todo!(),
            TypeCheckerError::UnknownEnumValue {
                enum_name: _,
                enum_value: _,
            } => todo!(),
            TypeCheckerError::EnumInitArgCountMismatch(_, _) => todo!(),
        }

        Ok(())
    }
}
