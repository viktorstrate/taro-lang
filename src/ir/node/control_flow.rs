use crate::{ir::late_init::LateInit, parser::Span};

use super::{expression::Expr, identifier::Ident, statement::StmtBlock, NodeRef};

#[derive(Debug, Clone)]
pub struct IfStmt<'a> {
    pub condition: NodeRef<'a, Expr<'a>>,
    pub span: Span<'a>,
    pub body: NodeRef<'a, StmtBlock<'a>>,
    pub else_body: Option<NodeRef<'a, StmtBlock<'a>>>,
    pub body_scope_ident: LateInit<Ident<'a>>,
    pub else_scope_ident: LateInit<Ident<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfBranchBody {
    MainBody,
    ElseBody,
}

impl<'a> IfStmt<'a> {
    pub fn branch_ident(&self, branch: IfBranchBody) -> Ident<'a> {
        match branch {
            IfBranchBody::MainBody => *self.body_scope_ident,
            IfBranchBody::ElseBody => *self.else_scope_ident,
        }
    }
}
