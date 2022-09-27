use crate::parser::Span;

use super::{expression::Expr, identifier::Ident};

#[derive(Debug, Clone)]
pub struct MemberAccess<'a> {
    pub object: Option<Expr<'a>>,
    pub member_name: Ident<'a>,
    pub items: Option<(Span<'a>, Vec<Expr<'a>>)>,
}
