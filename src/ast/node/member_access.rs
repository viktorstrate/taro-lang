use super::{expression::Expr, identifier::Ident};

#[derive(Debug, Clone)]
pub struct MemberAccess<'a> {
    pub object: Option<Expr<'a>>,
    pub member_name: Ident<'a>,
    pub items: Option<Vec<Expr<'a>>>,
}
