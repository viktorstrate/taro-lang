use std::{fmt::Debug};

use super::{
    node::{
        enumeration::Enum,
        expression::Expr,
        function::Function,
        module::Module,
        statement::Stmt,
        structure::{Struct, StructInit},
    },
    AST,
};

pub enum ScopeValue<'a, 'ctx, 'scp> {
    Func(&'scp mut Function<'a, 'ctx>),
    Struct(&'scp mut Struct<'a, 'ctx>),
    StructInit(&'scp mut StructInit<'a, 'ctx>),
    Enum(&'scp mut Enum<'a, 'ctx>),
}

#[allow(unused_variables)]
pub trait AstWalker<'a, 'ctx> {
    type Scope: Default = ();
    type Error: Debug = ();

    fn visit_begin(&mut self, scope: &mut Self::Scope) -> Result<(), Self::Error> {
        Ok(())
    }

    fn pre_visit_stmt(
        &mut self,
        scope: &mut Self::Scope,
        stmt: &mut Stmt<'a, 'ctx>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_stmt(
        &mut self,
        scope: &mut Self::Scope,
        stmt: &mut Stmt<'a, 'ctx>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_scope_begin(
        &mut self,
        parent: &mut Self::Scope,
        value: ScopeValue<'a, 'ctx, '_>,
    ) -> Result<Self::Scope, Self::Error> {
        Ok(Self::Scope::default())
    }

    fn visit_scope_end(
        &mut self,
        parent: &mut Self::Scope,
        child: Self::Scope,
        value: ScopeValue<'a, 'ctx, '_>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_expr(
        &mut self,
        scope: &mut Self::Scope,
        expr: &mut Expr<'a, 'ctx>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn walk_ast<'a, 'ctx, W: AstWalker<'a, 'ctx>>(
    walker: &mut W,
    ast: &mut AST<'a, 'ctx>,
) -> Result<W::Scope, W::Error> {
    let mut global_scope = W::Scope::default();
    walker.visit_begin(&mut global_scope)?;
    walk_module(walker, &mut global_scope, &mut ast.0)?;
    Ok(global_scope)
}

fn walk_module<'a, 'ctx, W: AstWalker<'a, 'ctx>>(
    walker: &mut W,
    scope: &mut W::Scope,
    module: &mut Module<'a, 'ctx>,
) -> Result<(), W::Error> {
    for stmt in &mut module.stmts {
        walk_stmt(walker, scope, stmt)?;
    }

    Ok(())
}

fn walk_struct<'a, 'ctx, W: AstWalker<'a, 'ctx>>(
    walker: &mut W,
    scope: &mut W::Scope,
    st: &mut Struct<'a, 'ctx>,
) -> Result<(), W::Error> {
    let mut st_scope = walker.visit_scope_begin(scope, ScopeValue::Struct(st))?;

    for attr in &mut st.attrs {
        if let Some(value) = &mut attr.default_value {
            walk_expr(walker, &mut st_scope, value)?;
        }
    }

    walker.visit_scope_end(scope, st_scope, ScopeValue::Struct(st))?;

    Ok(())
}

fn walk_enum<'a, 'ctx, W: AstWalker<'a, 'ctx>>(
    walker: &mut W,
    scope: &mut W::Scope,
    enm: &mut Enum<'a, 'ctx>,
) -> Result<(), W::Error> {
    let enm_scope = walker.visit_scope_begin(scope, ScopeValue::Enum(enm))?;
    walker.visit_scope_end(scope, enm_scope, ScopeValue::Enum(enm))?;

    Ok(())
}

fn walk_stmt<'a, 'ctx, W: AstWalker<'a, 'ctx>>(
    walker: &mut W,
    scope: &mut W::Scope,
    stmt: &mut Stmt<'a, 'ctx>,
) -> Result<(), W::Error> {
    walker.pre_visit_stmt(scope, stmt)?;
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
        Stmt::EnumDecl(enm) => walk_enum(walker, scope, enm)?,
        Stmt::Return(expr) => walk_expr(walker, scope, expr)?,
    };
    walker.visit_stmt(scope, stmt)?;
    Ok(())
}

fn walk_func_decl<'a, 'ctx, W: AstWalker<'a, 'ctx>>(
    walker: &mut W,
    scope: &mut W::Scope,
    func: &mut Function<'a, 'ctx>,
) -> Result<(), W::Error> {
    let mut func_scope = walker.visit_scope_begin(scope, ScopeValue::Func(func))?;
    walk_stmt(walker, &mut func_scope, &mut func.body)?;
    walker.visit_scope_end(scope, func_scope, ScopeValue::Func(func))?;

    Ok(())
}

fn walk_expr<'a, 'ctx, W: AstWalker<'a, 'ctx>>(
    walker: &mut W,
    scope: &mut W::Scope,
    expr: &mut Expr<'a, 'ctx>,
) -> Result<(), W::Error> {
    match expr {
        Expr::Function(func) => walk_func_decl(walker, scope, func),
        Expr::Assignment(asg) => {
            walk_expr(walker, scope, asg.lhs)?;
            walk_expr(walker, scope, asg.rhs)
        }
        Expr::StructAccess(st_access) => walk_expr(walker, scope, st_access.struct_expr),
        Expr::StructInit(st_init) => walk_struct_init(walker, scope, st_init),
        _ => Ok(()),
    }?;

    walker.visit_expr(scope, expr)
}

fn walk_struct_init<'a, 'ctx, W: AstWalker<'a, 'ctx>>(
    walker: &mut W,
    scope: &mut W::Scope,
    st_init: &mut StructInit<'a, 'ctx>,
) -> Result<(), W::Error> {
    let mut child_scope = walker.visit_scope_begin(scope, ScopeValue::StructInit(st_init))?;
    for value in &mut st_init.values {
        walk_expr(walker, &mut child_scope, &mut value.value)?;
    }
    walker.visit_scope_end(scope, child_scope, ScopeValue::StructInit(st_init))?;
    Ok(())
}
