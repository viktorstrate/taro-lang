use std::fmt::Debug;

use id_arena::Id;

use super::{
    context::IrCtx,
    node::{
        enumeration::Enum,
        expression::Expr,
        function::Function,
        module::Module,
        statement::Stmt,
        structure::{Struct, StructInit},
    },
    IR,
};

pub enum ScopeValue<'a> {
    Func(Id<Function<'a>>),
    Struct(Id<Struct<'a>>),
    StructInit(Id<StructInit<'a>>),
    Enum(Id<Enum<'a>>),
}

#[allow(unused_variables)]
pub trait AstWalker<'a> {
    type Scope: Default = ();
    type Error: Debug = ();

    fn visit_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn pre_visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        stmt: Id<Stmt<'a>>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        stmt: Id<Stmt<'a>>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        parent: &mut Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<Self::Scope, Self::Error> {
        Ok(Self::Scope::default())
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        parent: &mut Self::Scope,
        child: Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        expr: Id<Expr<'a>>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn walk_ast<'a, 'ctx, W: AstWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    ast: &mut IR<'a>,
) -> Result<W::Scope, W::Error> {
    let mut global_scope = W::Scope::default();
    walker.visit_begin(ctx, &mut global_scope)?;
    walk_module(walker, ctx, &mut global_scope, &mut ast.0)?;
    Ok(global_scope)
}

fn walk_module<'a, W: AstWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    module: &mut Module<'a>,
) -> Result<(), W::Error> {
    for stmt in &mut module.stmts {
        walk_stmt(walker, ctx, scope, *stmt)?;
    }

    Ok(())
}

fn walk_struct<'a, W: AstWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    st: Id<Struct<'a>>,
) -> Result<(), W::Error> {
    let mut st_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Struct(st))?;

    for attr_id in ctx.nodes.st_decls[st].attrs.clone() {
        let attr = &mut ctx.nodes.st_attrs[attr_id];
        match attr.default_value {
            Some(value) => {
                walk_expr(walker, ctx, &mut st_scope, value)?;
            }
            _ => (),
        }
    }

    walker.visit_scope_end(ctx, scope, st_scope, ScopeValue::Struct(st))?;

    Ok(())
}

fn walk_enum<'a, W: AstWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    enm: Id<Enum<'a>>,
) -> Result<(), W::Error> {
    let enm_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Enum(enm))?;
    walker.visit_scope_end(ctx, scope, enm_scope, ScopeValue::Enum(enm))?;

    Ok(())
}

fn walk_stmt<'a, W: AstWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    stmt: Id<Stmt<'a>>,
) -> Result<(), W::Error> {
    walker.pre_visit_stmt(ctx, scope, stmt)?;
    let stmt_val = &ctx.nodes.stmts[stmt];
    match stmt_val {
        Stmt::VariableDecl(decl) => {
            let decl = *decl;
            walk_expr(walker, ctx, scope, ctx.nodes.var_decls[decl].value)?
        }
        Stmt::Expression(expr) => {
            let expr = *expr;
            walk_expr(walker, ctx, scope, expr)?
        }
        Stmt::FunctionDecl(func) => {
            let func = *func;
            walk_func_decl(walker, ctx, scope, func)?
        }
        Stmt::Compound(stmts) => {
            for stmt in stmts.clone() {
                walk_stmt(walker, ctx, scope, stmt)?;
            }
        }
        Stmt::StructDecl(st) => {
            let st = *st;
            walk_struct(walker, ctx, scope, st)?
        }
        Stmt::EnumDecl(enm) => {
            let enm = *enm;
            walk_enum(walker, ctx, scope, enm)?
        }
        Stmt::Return(expr) => {
            let expr = *expr;
            walk_expr(walker, ctx, scope, expr)?
        }
    };
    walker.visit_stmt(ctx, scope, stmt)?;
    Ok(())
}

fn walk_func_decl<'a, 'ctx, W: AstWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    func: Id<Function<'a>>,
) -> Result<(), W::Error> {
    let mut func_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Func(func))?;
    walk_stmt(walker, ctx, &mut func_scope, ctx.nodes.funcs[func].body)?;
    walker.visit_scope_end(ctx, scope, func_scope, ScopeValue::Func(func))?;

    Ok(())
}

fn walk_expr<'a, 'ctx, W: AstWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    expr: Id<Expr<'a>>,
) -> Result<(), W::Error> {
    match &ctx.nodes.exprs[expr] {
        &Expr::Function(func) => walk_func_decl(walker, ctx, scope, func),
        &Expr::Assignment(asg_id) => {
            let asg = &ctx.nodes.asgns[asg_id];
            let lhs = asg.lhs;
            let rhs = asg.rhs;
            walk_expr(walker, ctx, scope, lhs)?;
            walk_expr(walker, ctx, scope, rhs)
        }
        &Expr::StructAccess(st_access) => {
            walk_expr(walker, ctx, scope, ctx.nodes.st_accs[st_access].struct_expr)
        }
        &Expr::StructInit(st_init) => walk_struct_init(walker, ctx, scope, st_init),
        _ => Ok(()),
    }?;

    walker.visit_expr(ctx, scope, expr)
}

fn walk_struct_init<'a, 'ctx, W: AstWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    st_init: Id<StructInit<'a>>,
) -> Result<(), W::Error> {
    let mut child_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::StructInit(st_init))?;
    for value in ctx.nodes.st_inits[st_init].values.clone() {
        let expr = ctx.nodes.st_init_vals[value].value;
        walk_expr(walker, ctx, &mut child_scope, expr)?;
    }
    walker.visit_scope_end(ctx, scope, child_scope, ScopeValue::StructInit(st_init))?;
    Ok(())
}
