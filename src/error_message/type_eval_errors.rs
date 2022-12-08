use std::io::Write;

use crate::ir::{context::IrCtx, node::type_signature::TypeEvalError};

use super::{
    error_formatter::{format_span_items, ErrMsgType, SpanItem, Spanned},
    ErrMsg, ErrorMessage,
};

impl<'a: 'ret, 'ret, W: Write> ErrorMessage<'a, 'ret, &'ret IrCtx<'a>, W> for TypeEvalError<'a> {
    fn err_msg(&'ret self, ctx: &'ret IrCtx<'a>) -> ErrMsg<'a, 'ret, W> {
        match self {
            TypeEvalError::CallNonFunction(call, type_sig) => ErrMsg {
                span: type_sig.get_span(ctx),
                title: Box::new(|w| write!(w, "call non-function")),
                msg: Box::new(|w| {
                    if let Some(type_sig_span) = type_sig.get_span(ctx) {
                        format_span_items(
                            w,
                            &mut [
                                SpanItem {
                                    span: type_sig_span,
                                    msg: Some(format!(
                                        "type of called object is `{}`, expected function",
                                        type_sig.format(ctx)
                                    )),
                                    err_type: ErrMsgType::Err,
                                },
                                SpanItem {
                                    span: ctx[*call].args_span.clone(),
                                    msg: Some(format!("tried to call it here")),
                                    err_type: ErrMsgType::Err,
                                },
                            ],
                            &[],
                        )
                    } else {
                        format_span_items(
                            w,
                            &mut [SpanItem {
                                span: ctx[*call].func.get_span(ctx).unwrap(),
                                msg: Some(format!(
                                    "tried to call object of type `{}`, expected function",
                                    type_sig.format(ctx)
                                )),
                                err_type: ErrMsgType::Err,
                            }],
                            &[],
                        )
                    }
                }),
            },
            TypeEvalError::FuncWrongNumberOfArgs {
                func: _,
                expected: _,
                actual: _,
            } => todo!(),
            TypeEvalError::AccessNonStruct(_) => todo!(),
            TypeEvalError::AccessNonTuple(type_sig) => ErrMsg {
                span: type_sig.get_span(ctx),
                title: Box::new(|w| write!(w, "access non-tuple")),
                msg: Box::new(|w| {
                    format_span_items(
                        w,
                        &mut [SpanItem {
                            span: type_sig.get_span(ctx).unwrap(),
                            msg: Some(format!(
                                "type of accessed object is `{}`, expected tuple",
                                type_sig.format(ctx)
                            )),
                            err_type: ErrMsgType::Err,
                        }],
                        &[],
                    )
                }),
            },
            TypeEvalError::AccessNonEnum(_) => todo!(),
            TypeEvalError::TupleAccessOutOfBounds(tup_acc, tuple_type) => ErrMsg {
                span: tup_acc.get_span(ctx),
                title: Box::new(|w| write!(w, "tuple access index out of bounds")),
                msg: Box::new(move |w| {
                    let tuple_type_span = tuple_type
                        .get_span(ctx)
                        .unwrap_or(ctx[*tup_acc].tuple_expr.get_span(ctx).unwrap());

                    format_span_items(
                        w,
                        &mut [
                            SpanItem {
                                span: tuple_type_span,
                                msg: Some(format!("tuple of type `{}`", tuple_type.format(ctx))),
                                err_type: ErrMsgType::Err,
                            },
                            SpanItem {
                                span: ctx[*tup_acc].span.clone(),
                                msg: Some(format!("has no index {}", ctx[*tup_acc].attr)),
                                err_type: ErrMsgType::Err,
                            },
                        ],
                        &[],
                    )
                }),
            },
            TypeEvalError::UnknownIdent(id) => ErrMsg {
                span: id.get_span(ctx),
                title: Box::new(|w| write!(w, "unknown identifier `{}`", id.value(ctx).unwrap())),
                msg: Box::new(|w| {
                    format_span_items(
                        w,
                        &mut [SpanItem {
                            span: id.get_span(ctx).unwrap(),
                            msg: Some("identifier unknown".to_owned()),
                            err_type: ErrMsgType::Err,
                        }],
                        &[],
                    )
                }),
            },
        }
    }
}
