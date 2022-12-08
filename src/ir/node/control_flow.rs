use crate::{
    ir::{ast_lowering::IrLowerable, late_init::LateInit},
    parser::Span,
};

use super::{
    expression::Expr,
    identifier::{Ident, IdentParent},
    statement::StmtBlock,
    IrAlloc, NodeRef,
};

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

impl<'a> IrLowerable<'a> for crate::ast::node::control_flow::IfStmt<'a> {
    type IrType = IfStmt<'a>;

    fn ir_lower(self, ctx: &mut crate::ir::context::IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let condition = self.condition.ir_lower(ctx);
        let body = self.body.ir_lower(ctx);
        let else_body = self.else_body.map(|bdy| bdy.ir_lower(ctx));

        let if_branch = IfStmt {
            condition,
            body,
            else_body,
            span: self.span,
            body_scope_ident: LateInit::empty(),
            else_scope_ident: LateInit::empty(),
        }
        .allocate(ctx);

        ctx[if_branch].body_scope_ident = ctx
            .make_anon_ident(IdentParent::IfBranchScope(if_branch))
            .into();

        ctx[if_branch].else_scope_ident = ctx
            .make_anon_ident(IdentParent::IfBranchScope(if_branch))
            .into();

        if_branch
    }
}
