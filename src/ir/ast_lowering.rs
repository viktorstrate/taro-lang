use crate::{ast::AST, ir::node::statement::Stmt};

use super::{
    context::{IrArenaType, IrCtx},
    node::{module::Module, statement::StmtBlock, IrAlloc, NodeRef},
    IR,
};

#[derive(Debug)]
pub struct LowerAstResult<'a> {
    pub ctx: IrCtx<'a>,
    pub ir: IR<'a>,
}

pub fn lower_ast<'a>(ast: AST<'a>) -> LowerAstResult<'a> {
    let mut ctx = IrCtx::new();
    let module = ast.0;

    let stmt_block = ctx.lower_stmt(module.stmt);

    LowerAstResult {
        ctx,
        ir: IR(Module { stmt_block }),
    }
}

/// A trait that allows an AST node to be converted into an IR node
pub trait IrLowerable<'a> {
    type IrType: IrArenaType<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType>;
}

impl<'a> IrLowerable<'a> for crate::ast::node::statement::Stmt<'a> {
    type IrType = StmtBlock<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        ctx.lower_stmt(self)
    }
}

impl<'a> IrCtx<'a> {
    fn lower_stmt(
        &mut self,
        stmt: crate::ast::node::statement::Stmt<'a>,
    ) -> NodeRef<'a, StmtBlock<'a>> {
        fn unfold_stmts<'a>(
            ctx: &mut IrCtx<'a>,
            stmt: crate::ast::node::statement::Stmt<'a>,
            acc: &mut Vec<NodeRef<'a, Stmt<'a>>>,
        ) {
            match stmt.value {
                crate::ast::node::statement::StmtValue::VariableDecl(var_decl) => {
                    acc.push(Stmt::VariableDecl(var_decl.ir_lower(ctx)).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::FunctionDecl(func_decl) => {
                    acc.push(Stmt::FunctionDecl(func_decl.ir_lower(ctx)).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::StructDecl(st_decl) => {
                    acc.push(Stmt::StructDecl(st_decl.ir_lower(ctx)).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::EnumDecl(enm) => {
                    acc.push(Stmt::EnumDecl(enm.ir_lower(ctx)).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::Compound(stmts) => {
                    for stmt in stmts {
                        unfold_stmts(ctx, stmt, acc);
                    }
                }
                crate::ast::node::statement::StmtValue::Expression(expr) => {
                    acc.push(Stmt::Expression(expr.ir_lower(ctx)).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::Return(expr) => {
                    acc.push(Stmt::Return(expr.ir_lower(ctx)).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::Comment(_) => {}
                crate::ast::node::statement::StmtValue::ExternObj(ast_obj) => {
                    acc.push(Stmt::ExternObj(ast_obj.ir_lower(ctx)).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::IfBranch(ifb) => {
                    acc.push(Stmt::IfBranch(ifb.ir_lower(ctx)).allocate(ctx))
                }
                crate::ast::node::statement::StmtValue::TraitDecl(tr) => {
                    acc.push(Stmt::TraitDecl(tr.ir_lower(ctx)).allocate(ctx));
                }
            };
        }

        let mut stmt_buffer = Vec::new();
        unfold_stmts(self, stmt, &mut stmt_buffer);

        StmtBlock(stmt_buffer).allocate(self)
    }
}
