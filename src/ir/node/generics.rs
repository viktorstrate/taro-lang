use crate::{error_message::error_formatter::Spanned, parser::Span};

use super::{identifier::Ident, NodeRef};

#[derive(Debug, Clone)]
pub struct GenericsDecl<'a> {
    pub generics: Vec<Ident<'a>>,
    pub span: Span<'a>,
}

impl<'a> Spanned<'a> for NodeRef<'a, GenericsDecl<'a>> {
    fn get_span(&self, ctx: &crate::ir::context::IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}
