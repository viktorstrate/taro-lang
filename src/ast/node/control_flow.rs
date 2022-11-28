use crate::parser::Span;

use super::{expression::Expr, statement::Stmt};

#[derive(Debug, Clone)]
pub struct IfStmt<'a> {
    pub condition: Expr<'a>,
    pub span: Span<'a>,
    pub body: Box<Stmt<'a>>,
    pub else_body: Option<Box<Stmt<'a>>>,
}
