use super::expression::Expr;

#[derive(Debug, Clone)]
pub struct Assignment<'a> {
    pub lhs: Expr<'a>,
    pub rhs: Expr<'a>,
}
