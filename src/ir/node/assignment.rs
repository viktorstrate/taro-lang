use super::expression::Expr;

#[derive(Debug)]
pub struct Assignment<'a, 'ctx> {
    pub lhs: &'ctx mut Expr<'a, 'ctx>,
    pub rhs: &'ctx mut Expr<'a, 'ctx>,
}
