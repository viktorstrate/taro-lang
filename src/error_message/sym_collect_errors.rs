use std::io::Write;

use crate::{
    ir::{context::IrCtx, node::identifier::Identifiable},
    symbols::symbol_table::SymbolCollectionError,
};

use super::{
    error_formatter::{format_span_items, ErrMsgType, SpanItem, Spanned},
    ErrMsg, ErrorMessage,
};

impl<'a: 'ret, 'ret, W: Write> ErrorMessage<'a, 'ret, &'ret IrCtx<'a>, W>
    for SymbolCollectionError<'a>
{
    fn err_msg(&'ret self, ctx: &'ret IrCtx<'a>) -> ErrMsg<'a, 'ret, W> {
        match self {
            SymbolCollectionError::SymbolAlreadyExistsInScope { new, existing } => ErrMsg {
                span: new.get_span(ctx),
                title: Box::new(|w| {
                    write!(
                        w,
                        "symbol '{}' already exists in scope",
                        new.value(ctx).unwrap()
                    )
                }),
                msg: Box::new(|w| {
                    let existing_name = ctx[*existing].name(ctx);

                    format_span_items(
                        w,
                        &mut [
                            SpanItem {
                                span: new.get_span(ctx).unwrap(),
                                msg: Some(
                                    "a symbol of this name has already been defined".to_owned(),
                                ),
                                err_type: ErrMsgType::Err,
                            },
                            SpanItem {
                                span: existing_name.get_span(ctx).unwrap(),
                                msg: Some("symbol was first declared here".to_owned()),
                                err_type: ErrMsgType::Text,
                            },
                        ],
                        &[],
                    )?;

                    Ok(())
                }),
            },
            SymbolCollectionError::ScopeNotFound(scp) => ErrMsg {
                span: scp.get_span(ctx),
                title: Box::new(|w| write!(w, "scope not found")),
                msg: Box::new(|_w| todo!()),
            },
        }
    }
}
