use crate::{ir::context::IrCtx, symbols::symbol_resolver::SymbolResolutionError};
use std::io::Write;

use super::{
    error_formatter::{format_span_items, ErrMsgType, ErrRemark, SpanItem, Spanned},
    error_msg_utils::lev_dist,
    ErrMsg, ErrorMessage,
};

impl<'a: 'ret, 'ret, W: Write> ErrorMessage<'a, 'ret, &'ret IrCtx<'a>, W>
    for SymbolResolutionError<'a>
{
    fn err_msg(&'ret self, ctx: &'ret IrCtx<'a>) -> ErrMsg<'a, 'ret, W> {
        match self {
            SymbolResolutionError::TypeEval(err) => err.err_msg(ctx),
            SymbolResolutionError::UnknownEnumValue { enm, enum_value } => ErrMsg {
                span: enum_value.get_span(ctx),
                title: Box::new(|w| {
                    write!(
                        w,
                        "unknown enum value '{}.{}'",
                        (*ctx[*enm].name).value(ctx).unwrap(),
                        enum_value.value(ctx).unwrap()
                    )
                }),
                msg: Box::new(|w| {
                    let mut remarks = Vec::new();

                    let val_name = enum_value.value(ctx).unwrap();
                    let mut suggestions = ctx[*enm]
                        .values
                        .iter()
                        .filter_map(|val| {
                            let sug = ctx[*val].name.value(ctx).unwrap();
                            let dist = lev_dist(val_name, sug);
                            if dist < 6 {
                                Some((dist, sug))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    suggestions.sort_by(|(a, _), (b, _)| a.cmp(b));

                    if !suggestions.is_empty() {
                        let mut help = "Did you mean?\n".to_owned();

                        for (_, sug) in suggestions.into_iter().take(5) {
                            help += "      - ";
                            help += sug;
                            help += "\n";
                        }

                        remarks.push(ErrRemark {
                            msg: help,
                            err_type: ErrMsgType::Hint,
                        });
                    }

                    format_span_items(
                        w,
                        &mut [
                            SpanItem {
                                span: (*ctx[*enm].name).get_span(ctx).unwrap(),
                                msg: Some("from enum declared here".to_owned()),
                                err_type: ErrMsgType::Err,
                            },
                            SpanItem {
                                span: enum_value.get_span(ctx).unwrap(),
                                msg: Some("this value was not found".to_owned()),
                                err_type: ErrMsgType::Err,
                            },
                        ],
                        &remarks,
                    )
                }),
            },
            SymbolResolutionError::InvalidMemberAccessType { mem_acc, obj_type } => ErrMsg {
                span: ctx[*mem_acc]
                    .object
                    .and_then(|obj| obj.get_span(ctx))
                    .or(mem_acc.get_span(ctx)),
                title: Box::new(|w| {
                    write!(
                        w,
                        "cannot access member value from type '{}'",
                        obj_type.format(ctx)
                    )
                }),
                msg: Box::new(|w| {
                    format_span_items(
                        w,
                        &mut [SpanItem {
                            span: ctx[*mem_acc]
                                .object
                                .and_then(|obj| obj.get_span(ctx))
                                .or(mem_acc.get_span(ctx))
                                .unwrap(),
                            msg: Some(format!("of type '{}'", obj_type.format(ctx))),
                            err_type: ErrMsgType::Err,
                        }],
                        &[],
                    )
                }),
            },
        }
    }
}
