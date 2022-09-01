use super::{expression::Expr, identifier::Ident, NodeRef};

#[derive(Debug, Clone)]
pub struct UnresolvedMemberAccess<'a> {
    pub object: Option<NodeRef<'a, Expr<'a>>>,
    pub member_name: Ident<'a>,
    pub items: Vec<NodeRef<'a, Expr<'a>>>,
}
