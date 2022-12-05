use std::fmt::Debug;

use crate::symbols::symbol_table::symbol_table_zipper::SymbolTableZipper;

use super::{
    ast_lowering::LowerAstResult,
    context::IrCtx,
    node::{
        control_flow::{IfBranchBody, IfStmt},
        enumeration::{Enum, EnumValue},
        expression::Expr,
        external::ExternalObject,
        function::{Function, FunctionArg},
        identifier::Ident,
        module::Module,
        statement::{Stmt, StmtBlock, VarDecl},
        structure::{Struct, StructInit},
        traits::{Trait, TraitFuncAttr},
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
    Trait(NodeRef<'a, Trait<'a>>),
}

impl<'a> ScopeValue<'a> {
    pub fn visit_scope_begin(&self, ctx: &IrCtx<'a>, symbols: &mut SymbolTableZipper<'a>) {
        let scope_ident = match self {
            ScopeValue::Func(func) => *ctx[*func].name,
            ScopeValue::Struct(st) => *ctx[*st].name,
            ScopeValue::StructInit(st_init) => *ctx[*st_init].scope_name,

            ScopeValue::Enum(enm) => *ctx[*enm].name,

            ScopeValue::IfBranch(ifb, branch) => ctx[*ifb].branch_ident(*branch),
            ScopeValue::Trait(tr) => *ctx[*tr].name,
        };

        symbols
            .enter_scope(ctx, scope_ident)
            .expect("scope should exist");
    }
}

pub trait IrWalkable<'a> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error>;
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
    module.stmt_block.walk(walker, ctx, scope)
}

impl<'a> IrWalkable<'a> for NodeRef<'a, Struct<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        let mut st_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Struct(self))?;

        for attr_id in ctx[self].attrs.clone() {
            let attr_name = *ctx[attr_id].name;
            walker.visit_ident(ctx, scope, attr_name)?;

            match ctx[attr_id].default_value {
                Some(value) => {
                    value.walk(walker, ctx, &mut st_scope)?;
                }
                _ => (),
            }

            ctx[attr_id].type_sig = ctx[attr_id]
                .type_sig
                .cloned()
                .walk(walker, ctx, scope)?
                .into();
        }

        let st_name = *ctx[self].name;
        walker.visit_ident(ctx, scope, st_name)?;

        walker.visit_scope_end(ctx, scope, st_scope, ScopeValue::Struct(self))?;

        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, Enum<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        let mut enm_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Enum(self))?;
        walker.visit_ident(ctx, scope, *ctx[self].name)?;

        for val in ctx[self].values.clone() {
            val.walk(walker, ctx, &mut enm_scope)?;
        }

        ctx[self].type_sig = ctx[self].type_sig.cloned().walk(walker, ctx, scope)?.into();

        walker.visit_scope_end(ctx, scope, enm_scope, ScopeValue::Enum(self))?;

        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, EnumValue<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        walker.visit_ident(ctx, scope, *ctx[self].name)?;

        for (i, type_sig) in (*ctx[self].items).clone().into_iter().enumerate() {
            ctx[self].items[i] = type_sig.walk(walker, ctx, scope)?;
        }

        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, StmtBlock<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        for stmt in ctx[self].0.clone() {
            stmt.walk(walker, ctx, scope)?;
        }
        walker.visit_stmt_block(ctx, scope, self)?;
        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, Stmt<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        match ctx[self].clone() {
            Stmt::VariableDecl(decl) => decl.walk(walker, ctx, scope),
            Stmt::Expression(expr) => expr.walk(walker, ctx, scope),
            Stmt::FunctionDecl(func) => func.walk(walker, ctx, scope),
            Stmt::StructDecl(st) => st.walk(walker, ctx, scope),
            Stmt::EnumDecl(enm) => enm.walk(walker, ctx, scope),
            Stmt::Return(expr) => expr.walk(walker, ctx, scope),
            Stmt::ExternObj(obj) => obj.walk(walker, ctx, scope),
            Stmt::IfBranch(ifb) => ifb.walk(walker, ctx, scope),
            Stmt::TraitDecl(tr_decl) => tr_decl.walk(walker, ctx, scope),
        }?;

        walker.visit_stmt(ctx, scope, self)?;

        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, VarDecl<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        let decl_name = *ctx[self].name;

        walker.visit_ordered_symbol(ctx, scope)?;
        walker.visit_ident(ctx, scope, decl_name)?;

        ctx[self].value.walk(walker, ctx, scope)?;
        ctx[self].type_sig = ctx[self].type_sig.cloned().walk(walker, ctx, scope)?.into();
        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, ExternalObject<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        walker.visit_ident(ctx, scope, *ctx[self].ident)?;
        ctx[self].type_sig = ctx[self].type_sig.cloned().walk(walker, ctx, scope)?.into();
        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, Function<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        let mut func_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Func(self))?;

        for arg in ctx[self].args.clone() {
            arg.walk(walker, ctx, &mut func_scope)?;
        }

        let func_name = *ctx[self].name;
        walker.visit_ident(ctx, scope, func_name)?;

        ctx[self].return_type = ctx[self]
            .return_type
            .cloned()
            .walk(walker, ctx, scope)?
            .into();

        ctx[self].body.walk(walker, ctx, &mut func_scope)?;

        walker.visit_func_decl(ctx, &mut func_scope, self)?;

        walker.visit_scope_end(ctx, scope, func_scope, ScopeValue::Func(self))?;

        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, FunctionArg<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        walker.visit_ident(ctx, scope, *ctx[self].name)?;
        ctx[self].type_sig = ctx[self].type_sig.cloned().walk(walker, ctx, scope)?.into();
        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, IfStmt<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        ctx[self].condition.walk(walker, ctx, scope)?;

        let mut if_main_scope = walker.visit_scope_begin(
            ctx,
            scope,
            ScopeValue::IfBranch(self, IfBranchBody::MainBody),
        )?;

        ctx[self].body.walk(walker, ctx, &mut if_main_scope)?;

        walker.visit_scope_end(
            ctx,
            scope,
            if_main_scope,
            ScopeValue::IfBranch(self, IfBranchBody::MainBody),
        )?;

        if let Some(else_body) = ctx[self].else_body {
            let mut if_else_scope = walker.visit_scope_begin(
                ctx,
                scope,
                ScopeValue::IfBranch(self, IfBranchBody::ElseBody),
            )?;

            else_body.walk(walker, ctx, &mut if_else_scope)?;

            walker.visit_scope_end(
                ctx,
                scope,
                if_else_scope,
                ScopeValue::IfBranch(self, IfBranchBody::ElseBody),
            )?;
        }

        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, Expr<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        match ctx[self].clone() {
            Expr::Function(func) => func.walk(walker, ctx, scope),
            Expr::Assignment(asg_id) => {
                ctx[asg_id].lhs.walk(walker, ctx, scope)?;
                ctx[asg_id].rhs.walk(walker, ctx, scope)
            }
            Expr::StructAccess(st_access) => {
                ctx[st_access].struct_expr.walk(walker, ctx, scope)?;

                let attr_name = ctx[st_access].attr_name;
                walker.visit_ident(ctx, scope, attr_name)
            }
            Expr::StructInit(st_init) => st_init.walk(walker, ctx, scope),
            Expr::Identifier(ident, _) => walker.visit_ident(ctx, scope, *ident),
            Expr::StringLiteral(_, _) => Ok(()),
            Expr::NumberLiteral(_, _) => Ok(()),
            Expr::BoolLiteral(_, _) => Ok(()),
            Expr::FunctionCall(func_call) => {
                for param in ctx[func_call].args.clone() {
                    param.walk(walker, ctx, scope)?;
                }

                ctx[func_call].func.walk(walker, ctx, scope)
            }
            Expr::TupleAccess(tup_acc) => ctx[tup_acc].tuple_expr.walk(walker, ctx, scope),
            Expr::EscapeBlock(esc_blk) => {
                ctx[esc_blk].type_sig = ctx[esc_blk]
                    .type_sig
                    .cloned()
                    .walk(walker, ctx, scope)?
                    .into();

                Ok(())
            }
            Expr::Tuple(tup) => {
                for expr in ctx[tup].values.clone() {
                    expr.walk(walker, ctx, scope)?;
                }

                ctx[tup].type_sig = ctx[tup].type_sig.cloned().walk(walker, ctx, scope)?.into();
                Ok(())
            }
            Expr::EnumInit(enm_init) => {
                for item in ctx[enm_init].items.clone() {
                    item.walk(walker, ctx, scope)?;
                }

                walker.visit_ident(ctx, scope, ctx[enm_init].enum_value)?;
                walker.visit_ident(ctx, scope, ctx[enm_init].enum_name)?;

                Ok(())
            }
            Expr::UnresolvedMemberAccess(mem_acc) => {
                if ctx[mem_acc].items.is_some() {
                    for item in ctx[mem_acc].items.clone().unwrap().0 {
                        item.walk(walker, ctx, scope)?;
                    }
                }

                if let Some(obj) = ctx[mem_acc].object {
                    obj.walk(walker, ctx, scope)?;
                }

                walker.visit_ident(ctx, scope, *ctx[mem_acc].member_name)?;

                Ok(())
            }
        }?;

        walker.visit_expr(ctx, scope, self)
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, StructInit<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        let mut child_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::StructInit(self))?;

        let scp_name = *ctx[self].scope_name;

        ctx[self].type_sig = ctx[self]
            .type_sig
            .cloned()
            .walk(walker, ctx, &mut child_scope)?
            .into();

        walker.visit_ident(ctx, &mut child_scope, scp_name)?;

        for value in ctx[self].values.clone() {
            let val_ident = *ctx[value].name;
            walker.visit_ident(ctx, &mut child_scope, val_ident)?;

            ctx[value].value.walk(walker, ctx, &mut child_scope)?;
        }
        walker.visit_scope_end(ctx, scope, child_scope, ScopeValue::StructInit(self))?;
        Ok(())
    }
}

impl<'a> IrWalkable<'a> for TypeSignature<'a> {
    type Output = TypeSignature<'a>;

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        let new_type_sig = match ctx[&self].clone() {
            TypeSignatureValue::Builtin(_) => self,
            TypeSignatureValue::Unresolved(ident) => {
                walker.visit_ident(ctx, scope, ident)?;
                self
            }
            TypeSignatureValue::Function { args, return_type } => {
                let mut new_args = Vec::with_capacity(args.len());
                for arg in &*args {
                    new_args.push(arg.clone().walk(walker, ctx, scope)?);
                }

                let new_return_type = return_type.cloned().walk(walker, ctx, scope)?;

                ctx.get_type_sig(
                    TypeSignatureValue::Function {
                        args: new_args.into(),
                        return_type: new_return_type.into(),
                    },
                    self.context,
                )
            }
            TypeSignatureValue::Struct { name } => {
                walker.visit_ident(ctx, scope, name)?;
                self
            }
            TypeSignatureValue::Enum { name } => {
                walker.visit_ident(ctx, scope, name)?;
                self
            }
            TypeSignatureValue::Tuple(types) => {
                let mut new_items = Vec::with_capacity(types.len());
                for item in &*types {
                    new_items.push(item.clone().walk(walker, ctx, scope)?);
                }

                ctx.get_type_sig(TypeSignatureValue::Tuple(new_items.into()), self.context)
            }
            TypeSignatureValue::TypeVariable(_) => self,
        };

        walker.visit_type_sig(ctx, scope, new_type_sig)
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, Trait<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        walker.visit_ident(ctx, scope, *ctx[self].name)?;

        let mut child_scope = walker.visit_scope_begin(ctx, scope, ScopeValue::Trait(self))?;

        for attr in ctx[self].attrs.clone() {
            attr.walk(walker, ctx, &mut child_scope)?;
        }

        walker.visit_scope_end(ctx, scope, child_scope, ScopeValue::Trait(self))?;

        Ok(())
    }
}

impl<'a> IrWalkable<'a> for NodeRef<'a, TraitFuncAttr<'a>> {
    type Output = ();

    fn walk<W: IrWalker<'a>>(
        self,
        walker: &mut W,
        ctx: &mut IrCtx<'a>,
        scope: &mut W::Scope,
    ) -> Result<Self::Output, W::Error> {
        walker.visit_ident(ctx, scope, *ctx[self].name)?;

        ctx[self].return_type = ctx[self]
            .return_type
            .clone()
            .map(|ret_type| ret_type.walk(walker, ctx, scope))
            .map_or(Ok(None), |val| val.map(Some))?;

        for arg in ctx[self].args.clone() {
            arg.walk(walker, ctx, scope)?;
        }

        Ok(())
    }
}
