use id_arena::Id;

use super::expression::Expr;

#[derive(Debug)]
pub struct Assignment<'a> {
    pub lhs: Id<Expr<'a>>,
    pub rhs: Id<Expr<'a>>,
}
