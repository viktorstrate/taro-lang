use super::{expression::Expr, NodeRef};

#[derive(Debug)]
pub struct Assignment<'a> {
    pub lhs: NodeRef<'a, Expr<'a>>,
    pub rhs: NodeRef<'a, Expr<'a>>,
}
