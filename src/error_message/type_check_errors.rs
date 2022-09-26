use std::io::Write;

use crate::{
    ir::{context::IrCtx, node::type_signature::TypeSignature},
    type_checker::{
        check_assignment::AssignmentError, check_struct::StructTypeError, FunctionError,
        TypeChecker, TypeCheckerError,
    },
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
                msg: Box::new(|w| {
                    let mut eq_groups: Vec<Vec<&TypeSignature<'a>>> = Vec::new();

                    for constraint in &type_checker.constraints {
                        let (a, b) = (&constraint.0, &constraint.1);
                        let found_group = eq_groups
                            .iter_mut()
                            .find(|types| types.iter().any(|t| **t == *a || **t == *b));

                        if let Some(group) = found_group {
                            group.append(&mut vec![a, b]);
                        } else {
                            eq_groups.push(vec![a, b]);
                        }
                    }

                    let mut items = eq_groups
                        .first()
                        .unwrap()
                        .into_iter()
                        .map(|t| SpanItem {
                            span: t.get_span(ctx).unwrap(),
                            msg: None,
                            err_type: ErrMsgType::Err,
                        })
                        .collect::<Vec<_>>();

                    items.sort();
                    items.dedup();

                    format_span_items(
                        w,
                        &mut items,
                        &[
                            ErrRemark {
                                msg: "the type could not be inferred for this symbol".to_owned(),
                                err_type: ErrMsgType::Text,
                            },
                            ErrRemark {
                                msg: "consider specifying an explicit type".to_owned(),
                                err_type: ErrMsgType::Hint,
                            },
                        ],
                    )
                }),
            },
            TypeCheckerError::TypeEval(err) => err.err_msg(ctx),
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
                AssignmentError::NotLValue(expr) => ErrMsg {
                    span: expr.get_span(ctx),
                    title: Box::new(|w| write!(w, "invalid assignment")),
                    msg: Box::new(|w| {
                        format_span_items(
                            w,
                            &mut [SpanItem {
                                span: expr.get_span(ctx).unwrap(),
                                msg: Some("expression is not an L-value".to_owned()),
                                err_type: ErrMsgType::Err,
                            }],
                            &[],
                        )
                    }),
                },
            },
            TypeCheckerError::StructError(st, st_err) => match st_err {
                StructTypeError::MissingAttribute(st_init, id) => {
                    let st_name = *ctx[*st].name;

                    ErrMsg {
                        span: id.get_span(ctx),
                        title: Box::new(move |w| {
                            write!(
                                w,
                                "missing attribute `{}` for struct `{}`",
                                id.value(ctx).unwrap(),
                                st_name.value(ctx).unwrap()
                            )
                        }),
                        msg: Box::new(|w| {
                            format_span_items(
                                w,
                                &mut [
                                    SpanItem {
                                        span: id.get_span(ctx).unwrap(),
                                        msg: Some("this attribute is missing".to_owned()),
                                        err_type: ErrMsgType::Err,
                                    },
                                    SpanItem {
                                        span: st_init.get_span(ctx).unwrap(),
                                        msg: Some(format!(
                                            "missing attribute `{}`",
                                            id.value(ctx).unwrap()
                                        )),
                                        err_type: ErrMsgType::Err,
                                    },
                                ],
                                &[ErrRemark {
                                    msg: format!("consider specifying a value for `{}`, or giving the attribute a default value", id.value(ctx).unwrap()),
                                    err_type: ErrMsgType::Hint,
                                }],
                            )
                        }),
                    }
                }
                StructTypeError::UnknownAttribute(_) => todo!(),
            },
            TypeCheckerError::FunctionError(func_err) => match func_err {
                FunctionError::ArgCountMismatch(_, _) => todo!(),
                FunctionError::FuncCallWrongArgAmount(_) => todo!(),
            },
            TypeCheckerError::EnumInitArgCountMismatch(_, _) => todo!(),
            TypeCheckerError::AnonymousEnumInitNonEnum(_, _) => todo!(),
            TypeCheckerError::SymbolResolutionError(sym_res_err) => sym_res_err.err_msg(ctx),
        }
    }
}
