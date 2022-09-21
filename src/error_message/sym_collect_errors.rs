use std::io::Write;

use crate::{
    ir::{context::IrCtx, node::identifier::Identifiable},
    symbols::symbol_table::SymbolCollectionError,
};

use super::{ErrMsg, ErrorMessage};

impl<'a: 'ret, 'ret, W: Write> ErrorMessage<'a, 'ret, &'ret IrCtx<'a>, W>
    for SymbolCollectionError<'a>
{
    fn err_msg(&'ret self, ctx: &'ret IrCtx<'a>) -> ErrMsg<'a, 'ret, W> {
        match self {
            SymbolCollectionError::SymbolAlreadyExistsInScope { new, existing } => ErrMsg {
                span: new.get_span(ctx),
                title: Box::new(|w| write!(w, "symbol already exists in scope")),
                msg: Box::new(|w| {
                    new.get_span(ctx).unwrap().format_spanned_code(
                        w,
                        Some("a symbol of this name has already been defined"),
                    )?;

                    let existing_name = ctx[*existing].name(ctx);

                    writeln!(w)?;

                    existing_name
                        .get_span(ctx)
                        .unwrap()
                        .format_spanned_code(w, Some("it was first declared here"))?;

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
