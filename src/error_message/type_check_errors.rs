use std::io::Write;

use crate::{
    ir::context::IrCtx,
    type_checker::{check_assignment::AssignmentError, TypeChecker, TypeCheckerError},
};

use super::{
    error_formatter::{format_span_items, ErrMsgType, ErrRemark, SpanItem, Spanned},
    ErrMsg, ErrorMessage,
};

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
                    let mut messages = vec![];

                    let a_fmt = a.format(ctx);
                    let b_fmt = b.format(ctx);

                    if let Some(span) = a.get_span(ctx) {
                        messages.push(SpanItem {
                            span,
                            msg: Some(format!("of type '{}'", a_fmt)),
                            err_type: ErrMsgType::Note,
                        });
                    }

                    if let Some(span) = b.get_span(ctx) {
                        messages.push(SpanItem {
                            span,
                            msg: Some(format!("of type '{}'", b_fmt)),
                            err_type: ErrMsgType::Note,
                        });
                    }

                    format_span_items(
                        w,
                        &mut messages,
                        &[ErrRemark {
                            msg: format!(
                                "expected type '{}'\n         found type '{}'",
                                a_fmt, b_fmt
                            ),
                            err_type: ErrMsgType::Note,
                        }],
                    )?;

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
            TypeCheckerError::AssignmentError(asg, err) => match err {
                AssignmentError::ImmutableAssignment(id) => ErrMsg {
                    span: id.get_span(ctx),
                    title: Box::new(|w| write!(w, "immutable variable assignment")),
                    msg: Box::new(|w| {
                        format_span_items(
                            w,
                            &mut [
                                SpanItem {
                                    span: id.get_span(ctx).unwrap(),
                                    msg: Some("immutable variable declaration".to_owned()),
                                    err_type: ErrMsgType::Text,
                                },
                                SpanItem {
                                    span: ctx[*asg].lhs.get_span(ctx).unwrap(),
                                    msg: Some("variable assignment".to_owned()),
                                    err_type: ErrMsgType::Text,
                                },
                            ],
                            &[ErrRemark {
                                msg: "change `let` declaration to `var` declaration".to_owned(),
                                err_type: ErrMsgType::Hint,
                            }],
                        )?;

                        Ok(())
                    }),
                },
                AssignmentError::NotLValue(_) => todo!(),
                AssignmentError::TypesMismatch { lhs: _, rhs: _ } => todo!(),
            },
            TypeCheckerError::StructError(_) => todo!(),
            TypeCheckerError::FunctionError(_) => todo!(),
            TypeCheckerError::EnumInitArgCountMismatch(_, _) => todo!(),
            TypeCheckerError::AnonymousEnumInitNonEnum(_, _) => todo!(),
            TypeCheckerError::SymbolResolutionError(sym_res_err) => sym_res_err.err_msg(ctx),
        }
    }
}
