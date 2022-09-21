use crate::{
    error_message::error_formatter::Spanned,
    ir::{context::IrCtx, late_init::LateInit},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    assignment::Assignment,
    enumeration::EnumInit,
    escape_block::EscapeBlock,
    function::{Function, FunctionCall},
    identifier::Ident,
    member_access::UnresolvedMemberAccess,
    structure::{StructAccess, StructInit},
    tuple::{Tuple, TupleAccess},
    type_signature::{
        BuiltinType, TypeEvalError, TypeSignature, TypeSignatureContext, TypeSignatureParent,
        TypeSignatureValue, Typed,
    },
    NodeRef,
};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    StringLiteral(&'a str, Span<'a>),
    NumberLiteral(f64, Span<'a>),
    BoolLiteral(bool, Span<'a>),
    Function(NodeRef<'a, Function<'a>>),
    FunctionCall(NodeRef<'a, FunctionCall<'a>>),
    Identifier(LateInit<Ident<'a>>, Span<'a>),
    StructInit(NodeRef<'a, StructInit<'a>>),
    StructAccess(NodeRef<'a, StructAccess<'a>>),
    TupleAccess(NodeRef<'a, TupleAccess<'a>>),
    EscapeBlock(NodeRef<'a, EscapeBlock<'a>>),
    Assignment(NodeRef<'a, Assignment<'a>>),
    Tuple(NodeRef<'a, Tuple<'a>>),
    EnumInit(NodeRef<'a, EnumInit<'a>>),
    UnresolvedMemberAccess(NodeRef<'a, UnresolvedMemberAccess<'a>>),
}

impl<'a> NodeRef<'a, Expr<'a>> {
    pub fn unwrap_func(self, ctx: &IrCtx<'a>) -> NodeRef<'a, Function<'a>> {
        match ctx[self] {
            Expr::Function(func) => func,
            _ => panic!("failed to unwrap expr as function"),
        }
    }
}

impl<'a> Typed<'a> for NodeRef<'a, Expr<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match ctx[*self].clone() {
            Expr::StringLiteral(_, _) => Ok(ctx.get_type_sig(
                TypeSignatureValue::Builtin(BuiltinType::String),
                TypeSignatureContext {
                    parent: TypeSignatureParent::Expr(*self),
                    type_span: None,
                }
                .alloc(),
            )),
            Expr::NumberLiteral(_, _) => Ok(ctx.get_type_sig(
                TypeSignatureValue::Builtin(BuiltinType::Number),
                TypeSignatureContext {
                    parent: TypeSignatureParent::Expr(*self),
                    type_span: None,
                }
                .alloc(),
            )),
            Expr::BoolLiteral(_, _) => Ok(ctx.get_type_sig(
                TypeSignatureValue::Builtin(BuiltinType::Boolean),
                TypeSignatureContext {
                    parent: TypeSignatureParent::Expr(*self),
                    type_span: None,
                }
                .alloc(),
            )),
            Expr::Function(func) => func.eval_type(symbols, ctx),
            Expr::FunctionCall(call) => call.eval_type(symbols, ctx),
            Expr::Identifier(ident, _) => {
                let sym_val = symbols
                    .lookup(ctx, *ident)
                    .ok_or(TypeEvalError::UnknownIdent(*ident))?;

                sym_val.eval_type(symbols, ctx)
            }
            Expr::StructInit(struct_init) => struct_init.eval_type(symbols, ctx),
            Expr::StructAccess(struct_access) => struct_access.eval_type(symbols, ctx),
            Expr::EscapeBlock(block) => block.eval_type(symbols, ctx),
            Expr::Assignment(asg) => {
                let rhs = ctx[asg].rhs;
                rhs.eval_type(symbols, ctx)
            }
            Expr::Tuple(tup) => tup.eval_type(symbols, ctx),
            Expr::TupleAccess(tup_acc) => tup_acc.eval_type(symbols, ctx),
            Expr::EnumInit(enm_init) => enm_init.eval_type(symbols, ctx),
            Expr::UnresolvedMemberAccess(mem_acc) => mem_acc.eval_type(symbols, ctx),
        }
    }

    fn specified_type(&self, ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        match ctx[*self].clone() {
            Expr::StringLiteral(_, _) => None,
            Expr::NumberLiteral(_, _) => None,
            Expr::BoolLiteral(_, _) => None,
            Expr::Function(func) => func.specified_type(ctx),
            Expr::FunctionCall(call) => call.specified_type(ctx),
            Expr::Identifier(_, _) => None,
            Expr::StructInit(st_init) => st_init.specified_type(ctx),
            Expr::StructAccess(_) => None,
            Expr::EscapeBlock(block) => block.specified_type(ctx),
            Expr::Assignment(_) => None,
            Expr::Tuple(tup) => tup.specified_type(ctx),
            Expr::TupleAccess(tup_acc) => tup_acc.specified_type(ctx),
            Expr::EnumInit(enm_init) => enm_init.specified_type(ctx),
            Expr::UnresolvedMemberAccess(mem_acc) => mem_acc.specified_type(ctx),
        }
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        match ctx[*self].clone() {
            Expr::StringLiteral(_, _) => Ok(()),
            Expr::NumberLiteral(_, _) => Ok(()),
            Expr::BoolLiteral(_, _) => Ok(()),
            Expr::Function(func) => func.specify_type(ctx, new_type),
            Expr::FunctionCall(call) => call.specify_type(ctx, new_type),
            Expr::Identifier(_, _) => Ok(()),
            Expr::StructInit(st_init) => st_init.specify_type(ctx, new_type),
            Expr::StructAccess(_) => Ok(()),
            Expr::EscapeBlock(block) => block.specify_type(ctx, new_type),
            Expr::Assignment(_) => Ok(()),
            Expr::Tuple(tup) => tup.specify_type(ctx, new_type),
            Expr::TupleAccess(tup_acc) => tup_acc.specify_type(ctx, new_type),
            Expr::EnumInit(enm_init) => enm_init.specify_type(ctx, new_type),
            Expr::UnresolvedMemberAccess(mem_acc) => mem_acc.specify_type(ctx, new_type),
        }
    }
}

impl<'a> Spanned<'a> for NodeRef<'a, Expr<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        match ctx[*self].clone() {
            Expr::StringLiteral(_, span) => Some(span),
            Expr::NumberLiteral(_, span) => Some(span),
            Expr::BoolLiteral(_, span) => Some(span),
            Expr::Function(_) => todo!(),
            Expr::FunctionCall(_) => todo!(),
            Expr::Identifier(_, span) => Some(span),
            Expr::StructInit(_) => todo!(),
            Expr::StructAccess(_) => todo!(),
            Expr::TupleAccess(_) => todo!(),
            Expr::EscapeBlock(_) => todo!(),
            Expr::Assignment(_) => todo!(),
            Expr::Tuple(_) => todo!(),
            Expr::EnumInit(_) => todo!(),
            Expr::UnresolvedMemberAccess(_) => todo!(),
        }
    }
}
