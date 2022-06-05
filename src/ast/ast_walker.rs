use std::fmt::Debug;

use super::{
    node::{
        expression::Expr, function::Function, module::Module, statement::Stmt, structure::Struct,
    },
    AST,
};

pub enum ScopeValue<'a, 'v> {
    Func(&'v mut Function<'a>),
    Struct(&'v mut Struct<'a>),
}

#[allow(unused_variables)]
pub trait AstWalker<'a> {
    type Scope: Default = ();
    type Error: Debug = ();

    fn visit_begin(&mut self, scope: &mut Self::Scope) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_stmt(
        &mut self,
        scope: &mut Self::Scope,
        stmt: &mut Stmt<'a>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_struct_decl(
        &mut self,
        scope: &mut Self::Scope,
        st: &mut Struct<'a>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_scope_begin(
        &mut self,
        parent: &mut Self::Scope,
        value: ScopeValue<'a, '_>,
    ) -> Result<Self::Scope, Self::Error> {
        Ok(Self::Scope::default())
    }

    fn visit_scope_end(
        &mut self,
        parent: &mut Self::Scope,
        child: Self::Scope,
        value: ScopeValue<'a, '_>,
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
    walker.visit_begin(&mut global_scope)?;
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
    scope: &mut W::Scope,
    st: &mut Struct<'a>,
) -> Result<(), W::Error> {
    walker.visit_struct_decl(scope, st)?;

    let mut st_scope = walker.visit_scope_begin(scope, ScopeValue::Struct(st))?;

    for attr in &mut st.attrs {
        if let Some(value) = &mut attr.default_value {
            walk_expr(walker, &mut st_scope, value)?;
        }
    }

    walker.visit_scope_end(scope, st_scope, ScopeValue::Struct(st))?;

    Ok(())
}

fn walk_stmt<'a, W: AstWalker<'a>>(
    walker: &mut W,
    scope: &mut W::Scope,
    stmt: &mut Stmt<'a>,
) -> Result<(), W::Error> {
    match stmt {
        Stmt::VariableDecl(decl) => walk_expr(walker, scope, &mut decl.value)?,
        Stmt::Expression(expr) => walk_expr(walker, scope, expr)?,
        Stmt::FunctionDecl(func) => walk_func_decl(walker, scope, func)?,
        Stmt::Compound(stmts) => {
            for stmt in stmts {
                walk_stmt(walker, scope, stmt)?;
            }
        }
        Stmt::StructDecl(st) => walk_struct(walker, scope, st)?,
        Stmt::Return(expr) => walk_expr(walker, scope, expr)?,
    };
    walker.visit_stmt(scope, stmt)?;
    Ok(())
}

fn walk_func_decl<'a, W: AstWalker<'a>>(
    walker: &mut W,
    scope: &mut W::Scope,
    func: &mut Function<'a>,
) -> Result<(), W::Error> {
    let mut func_scope = walker.visit_scope_begin(scope, ScopeValue::Func(func))?;
    walk_stmt(walker, &mut func_scope, &mut func.body)?;
    walker.visit_scope_end(scope, func_scope, ScopeValue::Func(func))?;

    Ok(())
}

fn walk_expr<'a, W: AstWalker<'a>>(
    walker: &mut W,
    scope: &mut W::Scope,
    expr: &mut Expr<'a>,
) -> Result<(), W::Error> {
    walker.visit_expr(expr)?;

    match expr {
        Expr::Function(func) => walk_func_decl(walker, scope, func),
        _ => Ok(()),
    }
}
