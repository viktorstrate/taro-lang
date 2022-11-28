use std::fmt::Debug;

use crate::symbols::symbol_table::symbol_table_zipper::SymbolTableZipper;

use super::{
    ast_lowering::LowerAstResult,
    context::IrCtx,
    node::{
        control_flow::{IfBranchBody, IfStmt},
        enumeration::Enum,
        expression::Expr,
        function::Function,
        identifier::Ident,
        module::Module,
        statement::{Stmt, StmtBlock},
        structure::{Struct, StructInit},
        type_signature::{TypeSignature, TypeSignatureValue},
        NodeRef,
    },
};

#[derive(Debug, Clone)]
pub enum ScopeValue<'a> {
    Func(NodeRef<'a, Function<'a>>),
    Struct(NodeRef<'a, Struct<'a>>),
    StructInit(NodeRef<'a, StructInit<'a>>),
    Enum(NodeRef<'a, Enum<'a>>),
    IfBranch(NodeRef<'a, IfStmt<'a>>, IfBranchBody),
}

impl<'a> ScopeValue<'a> {
    pub fn visit_scope_begin(&self, ctx: &IrCtx<'a>, symbols: &mut SymbolTableZipper<'a>) {
        let scope_ident = match self {
            ScopeValue::Func(func) => *ctx[*func].name,
            ScopeValue::Struct(st) => *ctx[*st].name,
            ScopeValue::StructInit(st_init) => *ctx[*st_init].scope_name,

            ScopeValue::Enum(enm) => *ctx[*enm].name,

            ScopeValue::IfBranch(ifb, branch) => ctx[*ifb].branch_ident(*branch),
        };

        symbols
            .enter_scope(ctx, scope_ident)
            .expect("scope should exist");
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

    fn visit_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_stmt_block(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        stmt_block: NodeRef<'a, StmtBlock<'a>>,
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

    fn visit_func_decl(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        func: NodeRef<'a, Function<'a>>,
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

    fn visit_ordered_symbol(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
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
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_type_sig(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
        type_sig: TypeSignature<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        Ok(type_sig)
    }
}

pub fn walk_ir<'a, W: IrWalker<'a>>(
    walker: &mut W,
    la: &mut LowerAstResult<'a>,
) -> Result<W::Scope, W::Error> {
    let mut global_scope = W::Scope::default();
    walker.visit_begin(&mut la.ctx, &mut global_scope)?;
    walk_module(walker, &mut la.ctx, &mut global_scope, &mut la.ir.0)?;
    walker.visit_end(&mut la.ctx, &mut global_scope)?;
    Ok(global_scope)
}

pub fn walk_module<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    module: &mut Module<'a>,
) -> Result<(), W::Error> {
    walk_stmt_block(walker, ctx, scope, module.stmt_block)
}

pub fn walk_struct<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    st: NodeRef<'a, Struct<'a>>,
) -> Result<(), W::Error> {
    let mut st_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Struct(st))?;

    for attr_id in ctx[st].attrs.clone() {
        let attr_name = *ctx[attr_id].name;
        walker.visit_ident(ctx, scope, attr_name)?;

        match ctx[attr_id].default_value {
            Some(value) => {
                walk_expr(walker, ctx, &mut st_scope, value)?;
            }
            _ => (),
        }

        ctx[attr_id].type_sig =
            walk_type_sig(walker, ctx, scope, (*ctx[attr_id].type_sig).clone())?.into();
    }

    let st_name = *ctx[st].name;
    walker.visit_ident(ctx, scope, st_name)?;

    walker.visit_scope_end(ctx, scope, st_scope, ScopeValue::Struct(st))?;

    Ok(())
}

pub fn walk_enum<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    enm: NodeRef<'a, Enum<'a>>,
) -> Result<(), W::Error> {
    let enm_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Enum(enm))?;
    walker.visit_ident(ctx, scope, *ctx[enm].name)?;

    for val in ctx[enm].values.clone() {
        let ident = *ctx[val].name;
        walker.visit_ident(ctx, scope, ident)?;

        for (i, type_sig) in (*ctx[val].items).clone().into_iter().enumerate() {
            ctx[val].items[i] = walk_type_sig(walker, ctx, scope, type_sig)?;
        }
    }

    ctx[enm].type_sig = walk_type_sig(walker, ctx, scope, (*ctx[enm].type_sig).clone())?.into();

    walker.visit_scope_end(ctx, scope, enm_scope, ScopeValue::Enum(enm))?;

    Ok(())
}

pub fn walk_stmt_block<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    stmt_block: NodeRef<'a, StmtBlock<'a>>,
) -> Result<(), W::Error> {
    for stmt in ctx[stmt_block].0.clone() {
        walk_stmt(walker, ctx, scope, stmt)?;
    }
    walker.visit_stmt_block(ctx, scope, stmt_block)?;
    Ok(())
}

pub fn walk_stmt<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    stmt: NodeRef<'a, Stmt<'a>>,
) -> Result<(), W::Error> {
    match ctx[stmt].clone() {
        Stmt::VariableDecl(decl) => {
            let decl_name = *ctx[decl].name;

            walker.visit_ordered_symbol(ctx, scope)?;
            walker.visit_ident(ctx, scope, decl_name)?;
            walk_expr(walker, ctx, scope, ctx[decl].value)?;

            ctx[decl].type_sig =
                walk_type_sig(walker, ctx, scope, (*ctx[decl].type_sig).clone())?.into();
        }
        Stmt::Expression(expr) => {
            walk_expr(walker, ctx, scope, expr)?;
        }
        Stmt::FunctionDecl(func) => {
            walk_func_decl(walker, ctx, scope, func)?;
        }
        Stmt::StructDecl(st) => {
            walk_struct(walker, ctx, scope, st)?;
        }
        Stmt::EnumDecl(enm) => {
            walk_enum(walker, ctx, scope, enm)?;
        }
        Stmt::Return(expr) => {
            walk_expr(walker, ctx, scope, expr)?;
        }
        Stmt::ExternObj(obj) => {
            walker.visit_ident(ctx, scope, *ctx[obj].ident)?;
            ctx[obj].type_sig =
                walk_type_sig(walker, ctx, scope, (*ctx[obj].type_sig).clone())?.into();
        }
        Stmt::IfBranch(ifb) => {
            walk_if_branch(walker, ctx, scope, ifb)?;
        }
    };
    walker.visit_stmt(ctx, scope, stmt)?;

    Ok(())
}

pub fn walk_func_decl<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    func: NodeRef<'a, Function<'a>>,
) -> Result<(), W::Error> {
    let mut func_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Func(func))?;

    for arg in ctx[func].args.clone() {
        let arg_name = *ctx[arg].name;
        walker.visit_ident(ctx, scope, arg_name)?;

        ctx[arg].type_sig = walk_type_sig(walker, ctx, scope, (*ctx[arg].type_sig).clone())?.into();
    }

    let func_name = *ctx[func].name;
    walker.visit_ident(ctx, scope, func_name)?;

    ctx[func].return_type =
        walk_type_sig(walker, ctx, scope, (*ctx[func].return_type).clone())?.into();

    walk_stmt_block(walker, ctx, &mut func_scope, ctx[func].body)?;

    walker.visit_func_decl(ctx, &mut func_scope, func)?;

    walker.visit_scope_end(ctx, scope, func_scope, ScopeValue::Func(func))?;

    Ok(())
}

pub fn walk_if_branch<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    ifb: NodeRef<'a, IfStmt<'a>>,
) -> Result<(), W::Error> {
    walk_expr(walker, ctx, scope, ctx[ifb].condition)?;

    let mut if_main_scope = walker.visit_scope_begin(
        ctx,
        scope,
        ScopeValue::IfBranch(ifb, IfBranchBody::MainBody),
    )?;
    walk_stmt_block(walker, ctx, &mut if_main_scope, ctx[ifb].body)?;
    walker.visit_scope_end(
        ctx,
        scope,
        if_main_scope,
        ScopeValue::IfBranch(ifb, IfBranchBody::MainBody),
    )?;

    if let Some(else_body) = ctx[ifb].else_body {
        let mut if_else_scope = walker.visit_scope_begin(
            ctx,
            scope,
            ScopeValue::IfBranch(ifb, IfBranchBody::ElseBody),
        )?;
        walk_stmt_block(walker, ctx, &mut if_else_scope, else_body)?;
        walker.visit_scope_end(
            ctx,
            scope,
            if_else_scope,
            ScopeValue::IfBranch(ifb, IfBranchBody::ElseBody),
        )?;
    }

    Ok(())
}

pub fn walk_expr<'a, W: IrWalker<'a>>(
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
        Expr::StructAccess(st_access) => {
            walk_expr(walker, ctx, scope, ctx[st_access].struct_expr)?;

            let attr_name = ctx[st_access].attr_name;
            walker.visit_ident(ctx, scope, attr_name)?;

            Ok(())
        }
        Expr::StructInit(st_init) => walk_struct_init(walker, ctx, scope, st_init),
        Expr::Identifier(ident, _) => {
            walker.visit_ident(ctx, scope, *ident)?;
            Ok(())
        }
        Expr::StringLiteral(_, _) => Ok(()),
        Expr::NumberLiteral(_, _) => Ok(()),
        Expr::BoolLiteral(_, _) => Ok(()),
        Expr::FunctionCall(func_call) => {
            for param in ctx[func_call].args.clone() {
                walk_expr(walker, ctx, scope, param)?;
            }

            let func_expr = ctx[func_call].func;
            walk_expr(walker, ctx, scope, func_expr)?;

            Ok(())
        }
        Expr::TupleAccess(tup_acc) => {
            let tup_expr = ctx[tup_acc].tuple_expr;
            walk_expr(walker, ctx, scope, tup_expr)?;
            Ok(())
        }
        Expr::EscapeBlock(esc_blk) => {
            ctx[esc_blk].type_sig =
                walk_type_sig(walker, ctx, scope, (*ctx[esc_blk].type_sig).clone())?.into();

            Ok(())
        }
        Expr::Tuple(tup) => {
            for expr in ctx[tup].values.clone() {
                walk_expr(walker, ctx, scope, expr)?;
            }

            ctx[tup].type_sig =
                walk_type_sig(walker, ctx, scope, (*ctx[tup].type_sig).clone())?.into();
            Ok(())
        }
        Expr::EnumInit(enm_init) => {
            for item in ctx[enm_init].items.clone() {
                walk_expr(walker, ctx, scope, item)?;
            }

            walker.visit_ident(ctx, scope, ctx[enm_init].enum_value)?;

            walker.visit_ident(ctx, scope, ctx[enm_init].enum_name)?;

            Ok(())
        }
        Expr::UnresolvedMemberAccess(mem_acc) => {
            if ctx[mem_acc].items.is_some() {
                for item in ctx[mem_acc].items.clone().unwrap().0 {
                    walk_expr(walker, ctx, scope, item)?;
                }
            }

            if let Some(obj) = ctx[mem_acc].object {
                walk_expr(walker, ctx, scope, obj)?;
            }

            walker.visit_ident(ctx, scope, *ctx[mem_acc].member_name)?;

            Ok(())
        }
    }?;

    walker.visit_expr(ctx, scope, expr)
}

pub fn walk_struct_init<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    st_init: NodeRef<'a, StructInit<'a>>,
) -> Result<(), W::Error> {
    let mut child_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::StructInit(st_init))?;

    let scp_name = *ctx[st_init].scope_name;

    ctx[st_init].type_sig = walk_type_sig(
        walker,
        ctx,
        &mut child_scope,
        (*ctx[st_init].type_sig).clone(),
    )?
    .into();

    walker.visit_ident(ctx, &mut child_scope, scp_name)?;

    for value in ctx[st_init].values.clone() {
        let val_ident = *ctx[value].name;
        walker.visit_ident(ctx, &mut child_scope, val_ident)?;

        let expr = ctx[value].value;
        walk_expr(walker, ctx, &mut child_scope, expr)?;
    }
    walker.visit_scope_end(ctx, scope, child_scope, ScopeValue::StructInit(st_init))?;
    Ok(())
}

pub fn walk_type_sig<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    type_sig: TypeSignature<'a>,
) -> Result<TypeSignature<'a>, W::Error> {
    let new_type_sig = match ctx[&type_sig].clone() {
        TypeSignatureValue::Builtin(_) => type_sig,
        TypeSignatureValue::Unresolved(ident) => {
            walker.visit_ident(ctx, scope, ident)?;
            type_sig
        }
        TypeSignatureValue::Function { args, return_type } => {
            let mut new_args = Vec::with_capacity(args.len());
            for arg in &*args {
                new_args.push(walk_type_sig(walker, ctx, scope, arg.clone())?);
            }

            let new_return_type = walk_type_sig(walker, ctx, scope, (*return_type).clone())?;

            ctx.get_type_sig(
                TypeSignatureValue::Function {
                    args: new_args.into(),
                    return_type: new_return_type.into(),
                },
                type_sig.context,
            )
        }
        TypeSignatureValue::Struct { name } => {
            walker.visit_ident(ctx, scope, name)?;
            type_sig
        }
        TypeSignatureValue::Enum { name } => {
            walker.visit_ident(ctx, scope, name)?;
            type_sig
        }
        TypeSignatureValue::Tuple(types) => {
            let mut new_items = Vec::with_capacity(types.len());
            for item in &*types {
                new_items.push(walk_type_sig(walker, ctx, scope, item.clone())?);
            }

            ctx.get_type_sig(
                TypeSignatureValue::Tuple(new_items.into()),
                type_sig.context,
            )
        }
        TypeSignatureValue::TypeVariable(_) => type_sig,
    };

    // let new_type_sig = ctx.get_type_sig(new_type_sig);
    walker.visit_type_sig(ctx, scope, new_type_sig)
}
