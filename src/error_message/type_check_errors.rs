use std::io::Write;

use crate::{
    ir::context::IrCtx,
    type_checker::{TypeChecker, TypeCheckerError},
};

use super::{ErrMsg, ErrorMessage};

impl<'a: 'ret, 'ret, W: Write> ErrorMessage<'a, 'ret, (&'ret TypeChecker<'a>, &'ret IrCtx<'a>), W>
    for TypeCheckerError<'a>
{
    fn err_msg(&'ret self, ctx: (&'ret TypeChecker<'a>, &'ret IrCtx<'a>)) -> ErrMsg<'a, 'ret, W> {
        let (type_checker, ctx) = ctx;

        match self {
            TypeCheckerError::ConflictingTypes(a, b) => ErrMsg {
                span: a.get_span(ctx).or_else(|| b.get_span(ctx)),
                title: Box::new(|w| write!(w, "conflicting types")),
                msg: Box::new(move |w| {
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

                    Ok(())
                }),
            },
            TypeCheckerError::UndeterminableTypes => ErrMsg {
                span: type_checker
                    .constraints
                    .front()
                    .unwrap()
                    .0
                    .clone()
                    .get_span(ctx),
                title: Box::new(|w| write!(w, "undeterminable types")),
                msg: Box::new(|_w| todo!()),
            },
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
    }
}
