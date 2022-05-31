use super::{Expr, Stmt, AST};

pub trait AstWalker<'a> {
    fn visit_expr(&mut self, expr: &Expr<'a>) {}
    fn visit_stmt(&mut self, stmt: &Stmt<'a>) {}
}

pub fn walk_ast<'a, W: AstWalker<'a>>(walker: &mut W, ast: &AST<'a>) {
    walk_stmt(walker, &ast.0)
}

fn walk_stmt<'a, W: AstWalker<'a>>(walker: &mut W, stmt: &Stmt<'a>) {
    walker.visit_stmt(&stmt);

    match stmt {
        Stmt::VarDecl(vardecl) => walk_expr(walker, &vardecl.value),
        Stmt::Compound(stmts) => {
            for stmt in stmts {
                walk_stmt(walker, stmt);
            }
        }
    }
}

fn walk_expr<'a, W: AstWalker<'a>>(walker: &mut W, expr: &Expr<'a>) {
    walker.visit_expr(expr);
}
