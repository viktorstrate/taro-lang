use crate::{error_message::error_formatter::Spanned, parser::Span};

use super::{expression::Expr, NodeRef};

#[derive(Debug)]
pub struct Assignment<'a> {
    pub lhs: NodeRef<'a, Expr<'a>>,
    pub rhs: NodeRef<'a, Expr<'a>>,
    pub span: Span<'a>,
}

impl<'a> Spanned<'a> for NodeRef<'a, Assignment<'a>> {
    fn get_span(&self, ctx: &crate::ir::context::IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}
