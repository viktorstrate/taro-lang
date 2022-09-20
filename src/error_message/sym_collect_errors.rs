use std::io::Write;

use crate::{
    ir::{context::IrCtx, node::identifier::Identifiable},
    parser::Span,
    symbols::symbol_table::SymbolCollectionError,
};

use super::ErrorMessage;

impl<'a, W: Write> ErrorMessage<'a, &IrCtx<'a>, W> for SymbolCollectionError<'a> {
    fn err_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        match self {
            SymbolCollectionError::SymbolAlreadyExistsInScope { new, existing: _ } => {
                new.get_span(ctx)
            }
            SymbolCollectionError::ScopeNotFound(scp) => scp.get_span(ctx),
        }
    }

    fn err_title(&self, w: &mut W, _ctx: &IrCtx<'a>) -> Result<(), std::io::Error> {
        match self {
            SymbolCollectionError::SymbolAlreadyExistsInScope {
                new: _,
                existing: _,
            } => write!(w, "symbol already exists in scope"),
            SymbolCollectionError::ScopeNotFound(_) => write!(w, "scope not found"),
        }
    }

    fn err_msg(&self, w: &mut W, ctx: &IrCtx<'a>) -> Result<(), std::io::Error> {
        match self {
            SymbolCollectionError::SymbolAlreadyExistsInScope { new, existing } => {
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
            }
            SymbolCollectionError::ScopeNotFound(_) => todo!(),
        }
    }
}
