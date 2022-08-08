use id_arena::Id;

use crate::{ir::context::IrCtx, symbols::symbol_table::symbol_table_zipper::SymbolTableZipper};

use super::{
    assignment::Assignment,
    escape_block::EscapeBlock,
    function::{Function, FunctionCall},
    identifier::Ident,
    structure::{StructAccess, StructInit},
    tuple::{Tuple, TupleAccess},
    type_signature::{BuiltinType, TypeEvalError, TypeSignature, Typed},
    NodeRef,
};

#[derive(Debug)]
pub enum Expr<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Function(NodeRef<'a, Function<'a>>),
    FunctionCall(NodeRef<'a, FunctionCall<'a>>),
    Identifier(Ident<'a>),
    StructInit(NodeRef<'a, StructInit<'a>>),
    StructAccess(NodeRef<'a, StructAccess<'a>>),
    TupleAccess(NodeRef<'a, TupleAccess<'a>>),
    EscapeBlock(NodeRef<'a, EscapeBlock<'a>>),
    Assignment(NodeRef<'a, Assignment<'a>>),
    Tuple(NodeRef<'a, Tuple<'a>>),
}

impl<'a> Typed<'a> for NodeRef<'a, Expr<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match &ctx[*self] {
            Expr::StringLiteral(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::String)),
            Expr::NumberLiteral(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Number)),
            Expr::BoolLiteral(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Boolean)),
            Expr::Function(func) => func.eval_type(symbols, ctx),
            Expr::FunctionCall(call) => call.eval_type(symbols, ctx),
            Expr::Identifier(ident) => {
                let sym_val = symbols
                    .lookup(ctx, *ident)
                    .ok_or(TypeEvalError::UnknownIdentifier(*ident))?;

                sym_val.eval_type(symbols, ctx)
            }
            Expr::StructInit(struct_init) => struct_init.eval_type(symbols, ctx),
            Expr::StructAccess(struct_access) => struct_access.eval_type(symbols, ctx),
            Expr::EscapeBlock(block) => block.eval_type(symbols, ctx),
            Expr::Assignment(asg) => ctx[*asg].rhs.eval_type(symbols, ctx),
            Expr::Tuple(tup) => tup.eval_type(symbols, ctx),
            Expr::TupleAccess(tup_acc) => tup_acc.eval_type(symbols, ctx),
        }
    }

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        let node_ref: &NodeRef<'a, dyn Typed<'a>> = match &ctx[*self] {
            Expr::StringLiteral(_) => None,
            Expr::NumberLiteral(_) => None,
            Expr::BoolLiteral(_) => None,
            Expr::Function(func) => func,
            Expr::FunctionCall(call) => call,
            Expr::Identifier(_) => None,
            Expr::StructInit(st_init) => st_init,
            Expr::StructAccess(_) => None,
            Expr::EscapeBlock(block) => block,
            Expr::Assignment(_) => None,
            Expr::Tuple(tup) => tup,
            Expr::TupleAccess(tup_acc) => tup_acc,
        }
    }

    fn specify_type(
        &mut self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        match &ctx[*self] {
            Expr::StringLiteral(_) => Ok(()),
            Expr::NumberLiteral(_) => Ok(()),
            Expr::BoolLiteral(_) => Ok(()),
            Expr::Function(func) => func.specify_type(ctx, new_type),
            Expr::FunctionCall(call) => call.specify_type(ctx, new_type),
            Expr::Identifier(_) => Ok(()),
            Expr::StructInit(st_init) => st_init.specify_type(ctx, new_type),
            Expr::StructAccess(_) => Ok(()),
            Expr::EscapeBlock(block) => block.specify_type(ctx, new_type),
            Expr::Assignment(_) => Ok(()),
            Expr::Tuple(tup) => tup.specify_type(ctx, new_type),
            Expr::TupleAccess(tup_acc) => tup_acc.specify_type(ctx, new_type),
        }
    }
}
