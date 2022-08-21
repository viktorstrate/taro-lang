use std::fmt::Debug;

use crate::symbols::symbol_table::symbol_table_zipper::SymbolTableZipper;

use super::{
    context::IrCtx,
    node::{
        enumeration::Enum,
        expression::Expr,
        function::Function,
        identifier::{Ident, IdentParent},
        module::Module,
        statement::Stmt,
        structure::{Struct, StructInit},
        type_signature::{TypeSignature, TypeSignatureValue},
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

    fn visit_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        scope: &mut Self::Scope,
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
        parent: IdentParent<'a>,
        ident: Ident<'a>,
    ) -> Result<Ident<'a>, Self::Error> {
        Ok(ident)
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
    ctx: &mut IrCtx<'a>,
    ir: &mut IR<'a>,
) -> Result<W::Scope, W::Error> {
    let mut global_scope = W::Scope::default();
    walker.visit_begin(ctx, &mut global_scope)?;
    walk_module(walker, ctx, &mut global_scope, &mut ir.0)?;
    walker.visit_end(ctx, &mut global_scope)?;
    Ok(global_scope)
}

pub fn walk_module<'a, W: IrWalker<'a>>(
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

pub fn walk_struct<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    st: NodeRef<'a, Struct<'a>>,
) -> Result<(), W::Error> {
    let mut st_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Struct(st))?;

    for attr_id in ctx[st].attrs.clone() {
        let attr_name = ctx[attr_id].name;
        ctx[attr_id].name = walker.visit_ident(
            ctx,
            scope,
            IdentParent::StructDeclAttrName(attr_id),
            attr_name,
        )?;

        match ctx[attr_id].default_value {
            Some(value) => {
                walk_expr(walker, ctx, &mut st_scope, value)?;
            }
            _ => (),
        }

        ctx[attr_id].type_sig = walk_type_sig(walker, ctx, scope, ctx[attr_id].type_sig)?;
    }

    let st_name = ctx[st].name;
    ctx[st].name = walker.visit_ident(ctx, scope, IdentParent::StructDeclName(st), st_name)?;

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
    ctx[enm].name =
        walker.visit_ident(ctx, scope, IdentParent::EnumDeclName(enm), ctx[enm].name)?;

    for val in ctx[enm].values.clone() {
        let ident = ctx[val].name;
        ctx[val].name =
            walker.visit_ident(ctx, scope, IdentParent::EnumDeclValueName(val), ident)?;

        for (i, type_sig) in ctx[val].items.clone().into_iter().enumerate() {
            ctx[val].items[i] = walk_type_sig(walker, ctx, scope, type_sig)?;
        }
    }

    ctx[enm].type_sig = walk_type_sig(walker, ctx, scope, ctx[enm].type_sig)?;

    walker.visit_scope_end(ctx, scope, enm_scope, ScopeValue::Enum(enm))?;

    Ok(())
}

pub fn walk_stmt<'a, W: IrWalker<'a>>(
    walker: &mut W,
    ctx: &mut IrCtx<'a>,
    scope: &mut W::Scope,
    stmt: NodeRef<'a, Stmt<'a>>,
) -> Result<(), W::Error> {
    let stmt_val = &ctx[stmt];
    match stmt_val {
        Stmt::VariableDecl(decl) => {
            let decl = *decl;
            let decl_name = ctx[decl].name;

            walker.visit_ordered_symbol(ctx, scope)?;
            ctx[decl].name =
                walker.visit_ident(ctx, scope, IdentParent::VarDeclName(decl), decl_name)?;
            walk_expr(walker, ctx, scope, ctx[decl].value)?;

            ctx[decl].type_sig = walk_type_sig(walker, ctx, scope, ctx[decl].type_sig)?;
        }
        Stmt::Expression(expr) => {
            let expr = *expr;
            walk_expr(walker, ctx, scope, expr)?;
        }
        Stmt::FunctionDecl(func) => {
            let func = *func;
            walk_func_decl(walker, ctx, scope, func)?;
        }
        Stmt::Compound(stmts) => {
            for stmt in stmts.clone() {
                walk_stmt(walker, ctx, scope, stmt)?;
            }
        }
        Stmt::StructDecl(st) => {
            let st = *st;
            walk_struct(walker, ctx, scope, st)?;
        }
        Stmt::EnumDecl(enm) => {
            let enm = *enm;
            walk_enum(walker, ctx, scope, enm)?;
        }
        Stmt::Return(expr) => {
            let expr = *expr;
            walk_expr(walker, ctx, scope, expr)?;
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
        let arg_name = ctx[arg].name;
        ctx[arg].name =
            walker.visit_ident(ctx, scope, IdentParent::FuncDeclArgName(arg), arg_name)?;

        ctx[arg].type_sig = walk_type_sig(walker, ctx, scope, ctx[arg].type_sig)?;
    }

    let func_name = ctx[func].name;
    ctx[func].name = walker.visit_ident(ctx, scope, IdentParent::FuncDeclName(func), func_name)?;

    ctx[func].return_type = walk_type_sig(walker, ctx, scope, ctx[func].return_type)?;

    walk_stmt(walker, ctx, &mut func_scope, ctx[func].body)?;

    walker.visit_scope_end(ctx, scope, func_scope, ScopeValue::Func(func))?;

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
            ctx[st_access].attr_name = walker.visit_ident(
                ctx,
                scope,
                IdentParent::StructAccessAttrName(st_access),
                attr_name,
            )?;
            Ok(())
        }
        Expr::StructInit(st_init) => walk_struct_init(walker, ctx, scope, st_init),
        Expr::Identifier(ident) => {
            let new_ident = walker.visit_ident(ctx, scope, IdentParent::IdentExpr(expr), ident)?;
            match &mut ctx[expr] {
                Expr::Identifier(ident) => *ident = new_ident,
                _ => unreachable!(),
            }
            Ok(())
        }
        Expr::StringLiteral(_) => Ok(()),
        Expr::NumberLiteral(_) => Ok(()),
        Expr::BoolLiteral(_) => Ok(()),
        Expr::FunctionCall(func_call) => {
            for param in ctx[func_call].params.clone() {
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
            ctx[esc_blk].type_sig = walk_type_sig(walker, ctx, scope, ctx[esc_blk].type_sig)?;

            Ok(())
        }
        Expr::Tuple(tup) => {
            for expr in ctx[tup].values.clone() {
                walk_expr(walker, ctx, scope, expr)?;
            }

            ctx[tup].type_sig = walk_type_sig(walker, ctx, scope, ctx[tup].type_sig)?;
            Ok(())
        }
        Expr::EnumInit(enm_init) => {
            for item in ctx[enm_init].items.clone() {
                walk_expr(walker, ctx, scope, item)?;
            }

            ctx[enm_init].enum_value = walker.visit_ident(
                ctx,
                scope,
                IdentParent::EnumInitValueName(enm_init),
                ctx[enm_init].enum_value,
            )?;

            ctx[enm_init].enum_name = walker.visit_ident(
                ctx,
                scope,
                IdentParent::EnumInitEnumName(enm_init),
                ctx[enm_init].enum_name,
            )?;

            Ok(())
        }
        Expr::UnresolvedMemberAccess(mem_acc) => {
            for item in ctx[mem_acc].items.clone() {
                walk_expr(walker, ctx, scope, item)?;
            }

            if let Some(obj) = ctx[mem_acc].object {
                walk_expr(walker, ctx, scope, obj)?;
            }

            ctx[mem_acc].member_name = walker.visit_ident(
                ctx,
                scope,
                IdentParent::MemberAccessMemberName(mem_acc),
                ctx[mem_acc].member_name,
            )?;

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

    let st_name = ctx[st_init].struct_name;
    let scp_name = ctx[st_init].scope_name;

    ctx[st_init].struct_name = walker.visit_ident(
        ctx,
        &mut child_scope,
        IdentParent::StructInitStructName(st_init),
        st_name,
    )?;
    ctx[st_init].scope_name = walker.visit_ident(
        ctx,
        &mut child_scope,
        IdentParent::StructInitScopeName(st_init),
        scp_name,
    )?;

    for value in ctx[st_init].values.clone() {
        let val_ident = ctx[value].name;
        ctx[value].name = walker.visit_ident(
            ctx,
            &mut child_scope,
            IdentParent::StructInitValueName(value),
            val_ident,
        )?;

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
    let new_type_sig: TypeSignatureValue<'a> = match ctx[type_sig].clone() {
        TypeSignatureValue::Builtin(builtin) => TypeSignatureValue::Builtin(builtin),
        TypeSignatureValue::Unresolved(ident) => TypeSignatureValue::Unresolved(
            walker.visit_ident(ctx, scope, IdentParent::TypeSigName(type_sig), ident)?,
        ),
        TypeSignatureValue::Function { args, return_type } => {
            let mut new_args = Vec::with_capacity(args.len());
            for arg in args {
                new_args.push(walk_type_sig(walker, ctx, scope, arg)?);
            }

            let new_return_type = walk_type_sig(walker, ctx, scope, return_type)?;

            TypeSignatureValue::Function {
                args: new_args,
                return_type: new_return_type,
            }
        }
        TypeSignatureValue::Struct { name } => TypeSignatureValue::Struct {
            name: walker.visit_ident(ctx, scope, IdentParent::TypeSigName(type_sig), name)?,
        },
        TypeSignatureValue::Enum { name } => TypeSignatureValue::Enum {
            name: walker.visit_ident(ctx, scope, IdentParent::TypeSigName(type_sig), name)?,
        },
        TypeSignatureValue::Tuple(types) => {
            let mut new_items = Vec::with_capacity(types.len());
            for item in types {
                new_items.push(walk_type_sig(walker, ctx, scope, item)?);
            }

            TypeSignatureValue::Tuple(new_items)
        }
        TypeSignatureValue::TypeVariable(_) => return Ok(type_sig),
    };

    let new_type_sig = ctx.get_type_sig(new_type_sig);
    walker.visit_type_sig(ctx, scope, new_type_sig)
}
