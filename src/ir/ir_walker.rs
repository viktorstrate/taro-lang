use std::fmt::Debug;



use crate::symbols::symbol_table::symbol_table_zipper::SymbolTableZipper;

use super::{
    context::IrCtx,
    node::{
        enumeration::Enum,
        expression::Expr,
        function::Function,
        identifier::Ident,
        module::Module,
        statement::Stmt,
        structure::{Struct, StructInit},
        NodeRef,
    },
    IR,
};

#[derive(Debug, Clone)]
pub enum ScopeValue<'a> {
    Func(NodeRef<'a, Function<'a>>),
    Struct(NodeRef<'a, Struct<'a>>),
    StructInit(NodeRef<'a, StructInit<'a>>),
    Enum(NodeRef<'a, Enum<'a>>),
}

impl<'a> ScopeValue<'a> {
    pub fn visit_scope_begin(&self, ctx: &IrCtx<'a>, symbols: &mut SymbolTableZipper<'a>) {
        match self {
            ScopeValue::Func(func) => {
                symbols
                    .enter_scope(ctx, ctx[*func].name)
                    .expect("scope should exist");
            }
            ScopeValue::Struct(st) => {
                symbols
                    .enter_scope(ctx, ctx[*st].name)
                    .expect("scope should exist");
            }
            ScopeValue::StructInit(st_init) => symbols
                .enter_scope(ctx, ctx[*st_init].scope_name)
                .expect("scope should exist"),
            ScopeValue::Enum(enm) => symbols
                .enter_scope(ctx, ctx[*enm].name)
                .expect("scope should exist"),
        }
    }
}

#[allow(unused_variables)]
pub trait IrWalker<'a> {
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
        stmt: NodeRef<'a, Stmt<'a>>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        stmt: NodeRef<'a, Stmt<'a>>,
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
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_ident(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        ident: Ident<'a>,
    ) -> Result<Ident<'a>, Self::Error> {
        Ok(ident)
    }
}

pub fn walk_ast<'a, 'ctx, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    ast: &mut IR<'a>,
) -> Result<W::Scope, W::Error> {
    let mut global_scope = W::Scope::default();
    walker.visit_begin(ctx, &mut global_scope)?;
    walk_module(walker, ctx, &mut global_scope, &mut ast.0)?;
    Ok(global_scope)
}

fn walk_module<'a, W: IrWalker<'a>>(
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

fn walk_struct<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    st: NodeRef<'a, Struct<'a>>,
) -> Result<(), W::Error> {
    let mut st_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Struct(st))?;

    for attr_id in ctx[st].attrs.clone() {
        let attr_name = ctx[attr_id].name;
        ctx[attr_id].name = walker.visit_ident(ctx, scope, attr_name)?;

        let attr = &mut ctx[attr_id];
        match attr.default_value {
            Some(value) => {
                walk_expr(walker, ctx, &mut st_scope, value)?;
            }
            _ => (),
        }
    }

    let st_name = ctx[st].name;
    ctx[st].name = walker.visit_ident(ctx, scope, st_name)?;

    walker.visit_scope_end(ctx, scope, st_scope, ScopeValue::Struct(st))?;

    Ok(())
}

fn walk_enum<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    enm: NodeRef<'a, Enum<'a>>,
) -> Result<(), W::Error> {
    let enm_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Enum(enm))?;
    ctx[enm].name = walker.visit_ident(ctx, scope, ctx[enm].name)?;

    for val in ctx[enm].values.clone() {
        let ident = ctx[val].name;
        ctx[val].name = walker.visit_ident(ctx, scope, ident)?;
    }

    walker.visit_scope_end(ctx, scope, enm_scope, ScopeValue::Enum(enm))?;

    Ok(())
}

fn walk_stmt<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    stmt: NodeRef<'a, Stmt<'a>>,
) -> Result<(), W::Error> {
    walker.pre_visit_stmt(ctx, scope, stmt)?;
    let stmt_val = &ctx[stmt];
    match stmt_val {
        Stmt::VariableDecl(decl) => {
            let decl = *decl;
            let decl_name = ctx[decl].name;
            ctx[decl].name = walker.visit_ident(ctx, scope, decl_name)?;
            walk_expr(walker, ctx, scope, ctx[decl].value)?
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

fn walk_func_decl<'a, 'ctx, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    func: NodeRef<'a, Function<'a>>,
) -> Result<(), W::Error> {
    let mut func_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Func(func))?;

    for arg in ctx[func].args.clone() {
        let arg_name = ctx[arg].name;
        ctx[arg].name = walker.visit_ident(ctx, scope, arg_name)?;
    }

    let func_name = ctx[func].name;
    ctx[func].name = walker.visit_ident(ctx, scope, func_name)?;

    walk_stmt(walker, ctx, &mut func_scope, ctx[func].body)?;

    walker.visit_scope_end(ctx, scope, func_scope, ScopeValue::Func(func))?;

    Ok(())
}

fn walk_expr<'a, 'ctx, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    expr: NodeRef<'a, Expr<'a>>,
) -> Result<(), W::Error> {
    match ctx[expr].clone() {
        Expr::Function(func) => walk_func_decl(walker, ctx, scope, func),
        Expr::Assignment(asg_id) => {
            let lhs = ctx[asg_id].lhs;
            let rhs = ctx[asg_id].rhs;
            walk_expr(walker, ctx, scope, lhs)?;
            walk_expr(walker, ctx, scope, rhs)
        }
        Expr::StructAccess(st_access) => walk_expr(walker, ctx, scope, ctx[st_access].struct_expr),
        Expr::StructInit(st_init) => walk_struct_init(walker, ctx, scope, st_init),
        Expr::Identifier(ident) => {
            let new_ident = walker.visit_ident(ctx, scope, ident)?;
            match &mut ctx[expr] {
                Expr::Identifier(ident) => *ident = new_ident,
                _ => unreachable!(),
            }
            Ok(())
        }
        Expr::StringLiteral(_) => Ok(()),
        Expr::NumberLiteral(_) => Ok(()),
        Expr::BoolLiteral(_) => Ok(()),
        Expr::FunctionCall(_) => Ok(()),
        Expr::TupleAccess(_) => Ok(()),
        Expr::EscapeBlock(_) => Ok(()),
        Expr::Tuple(_) => Ok(()),
    }?;

    walker.visit_expr(ctx, scope, expr)
}

fn walk_struct_init<'a, 'ctx, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    st_init: NodeRef<'a, StructInit<'a>>,
) -> Result<(), W::Error> {
    let mut child_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::StructInit(st_init))?;

    let st_name = ctx[st_init].struct_name;
    let scp_name = ctx[st_init].scope_name;

    ctx[st_init].struct_name = walker.visit_ident(ctx, scope, st_name)?;
    ctx[st_init].scope_name = walker.visit_ident(ctx, scope, scp_name)?;

    for value in ctx[st_init].values.clone() {
        let expr = ctx[value].value;
        walk_expr(walker, ctx, &mut child_scope, expr)?;
    }
    walker.visit_scope_end(ctx, scope, child_scope, ScopeValue::StructInit(st_init))?;
    Ok(())
}
