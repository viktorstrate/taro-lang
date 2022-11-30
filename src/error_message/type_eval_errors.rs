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
                    format_span_items(
                        w,
                        &mut [
                            SpanItem {
                                span: type_sig.get_span(ctx).unwrap(),
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
                }),
            },
            TypeEvalError::FuncWrongNumberOfArgs {
                func: _,
                expected: _,
                actual: _,
            } => todo!(),
            TypeEvalError::AccessNonStruct(_) => todo!(),
            TypeEvalError::AccessNonTuple(_) => todo!(),
            TypeEvalError::AccessNonEnum(_) => todo!(),
            TypeEvalError::TupleAccessOutOfBounds {
                tuple_len: _,
                access_item: _,
            } => todo!(),
            TypeEvalError::UnknownIdent(id) => ErrMsg {
                span: id.get_span(ctx),
                title: Box::new(|w| write!(w, "unknown identifier '{}'", id.value(ctx).unwrap())),
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