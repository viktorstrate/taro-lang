use crate::{
    ir::{ast_lowering::IrLowerable, context::IrCtx, late_init::LateInit},
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    control_flow::IfStmt,
    enumeration::Enum,
    expression::Expr,
    external::ExternalObject,
    function::Function,
    identifier::{Ident, IdentParent, Identifiable},
    structure::Struct,
    traits::Trait,
    type_signature::{Mutability, TypeEvalError, TypeSignature, TypeSignatureParent, Typed},
    IrAlloc, NodeRef,
};

#[derive(Debug, Clone)]
pub struct StmtBlock<'a>(pub Vec<NodeRef<'a, Stmt<'a>>>);

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    VariableDecl(NodeRef<'a, VarDecl<'a>>),
    FunctionDecl(NodeRef<'a, Function<'a>>),
    StructDecl(NodeRef<'a, Struct<'a>>),
    EnumDecl(NodeRef<'a, Enum<'a>>),
    TraitDecl(NodeRef<'a, Trait<'a>>),
    Expression(NodeRef<'a, Expr<'a>>),
    Return(NodeRef<'a, Expr<'a>>),
    ExternObj(NodeRef<'a, ExternalObject<'a>>),
    IfBranch(NodeRef<'a, IfStmt<'a>>),
}

#[derive(Debug)]
pub struct VarDecl<'a> {
    pub name: LateInit<Ident<'a>>,
    pub mutability: Mutability,
    pub type_sig: LateInit<TypeSignature<'a>>,
    pub value: NodeRef<'a, Expr<'a>>,
}

impl<'a> Identifiable<'a> for VarDecl<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.name
    }
}

impl<'a> Typed<'a> for NodeRef<'a, VarDecl<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        ctx[*self].value.clone().eval_type(symbols, ctx)
    }

    fn specified_type(&self, ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        Some((*ctx[*self].type_sig).clone())
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        ctx[*self].type_sig = new_type.into();
        Ok(())
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::statement::VarDecl<'a> {
    type IrType = VarDecl<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        let var_decl = VarDecl {
            name: LateInit::empty(),
            mutability: self.mutability,
            type_sig: LateInit::empty(),
            value: self.value.ir_lower(ctx),
        }
        .allocate(ctx);

        ctx[var_decl].name = ctx
            .make_ident(self.name, IdentParent::VarDeclName(var_decl))
            .into();

        ctx[var_decl].type_sig = self
            .type_sig
            .map(|type_sig| type_sig.into_ir_type(ctx, TypeSignatureParent::VarDeclSig(var_decl)))
            .unwrap_or_else(|| ctx.make_type_var(TypeSignatureParent::VarDeclSig(var_decl)))
            .into();

        var_decl
    }
}
