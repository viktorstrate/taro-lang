use std::fmt::Debug;

use super::{
    node::{
        expression::Expr, function::FuncDecl, identifier::Ident, module::Module, statement::Stmt,
        structure::Struct,
    },
    AST,
};

#[allow(unused_variables)]
pub trait AstWalker<'a> {
    type Scope: Default = ();
    type Error: Debug = ();

    fn visit_stmt(
        &mut self,
        scope: &mut Self::Scope,
        stmt: &mut Stmt<'a>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_struct_decl(&mut self, st: &mut Struct<'a>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_scope_begin(
        &mut self,
        parent: &mut Self::Scope,
        scope_ident: &Ident<'a>,
    ) -> Result<Self::Scope, Self::Error> {
        Ok(Self::Scope::default())
    }

    fn visit_scope_end(
        &mut self,
        parent: &mut Self::Scope,
        child: Self::Scope,
        scope_ident: &Ident<'a>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_expr(&mut self, expr: &mut Expr<'a>) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn walk_ast<'a, W: AstWalker<'a>>(
    walker: &mut W,
    ast: &mut AST<'a>,
) -> Result<W::Scope, W::Error> {
    let mut global_scope = W::Scope::default();
    walk_module(walker, &mut global_scope, &mut ast.0)?;
    Ok(global_scope)
}

fn walk_module<'a, W: AstWalker<'a>>(
    walker: &mut W,
    scope: &mut W::Scope,
    module: &mut Module<'a>,
) -> Result<(), W::Error> {
    for stmt in &mut module.stmts {
        walk_stmt(walker, scope, stmt)?;
    }

    Ok(())
}

fn walk_struct<'a, W: AstWalker<'a>>(
    walker: &mut W,
    _scope: &mut W::Scope,
    st: &mut Struct<'a>,
) -> Result<(), W::Error> {
    walker.visit_struct_decl(st)?;
    Ok(())
}

fn walk_stmt<'a, W: AstWalker<'a>>(
    walker: &mut W,
    scope: &mut W::Scope,
    stmt: &mut Stmt<'a>,
) -> Result<(), W::Error> {
    match stmt {
        Stmt::VariableDecl(decl) => walker.visit_expr(&mut decl.value)?,
        Stmt::Expression(expr) => walker.visit_expr(expr)?,
        Stmt::FunctionDecl(func) => walk_func_decl(walker, scope, func)?,
        Stmt::Compound(stmts) => {
            for stmt in stmts {
                walk_stmt(walker, scope, stmt)?;
            }
        }
        Stmt::StructDecl(st) => walk_struct(walker, scope, st)?,
        Stmt::Return(expr) => walker.visit_expr(expr)?,
    };
    walker.visit_stmt(scope, stmt)?;
    Ok(())
}

fn walk_func_decl<'a, W: AstWalker<'a>>(
    walker: &mut W,
    scope: &mut W::Scope,
    func: &mut FuncDecl<'a>,
) -> Result<(), W::Error> {
    let mut func_scope = walker.visit_scope_begin(scope, &func.name)?;
    walk_stmt(walker, &mut func_scope, &mut func.body)?;
    walker.visit_scope_end(scope, func_scope, &func.name)?;

    Ok(())
}
