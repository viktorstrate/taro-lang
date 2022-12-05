use crate::{
    error_message::error_formatter::Spanned,
    ir::{ast_lowering::IrLowerable, context::IrCtx, late_init::LateInit},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    assignment::Assignment,
    enumeration::EnumInit,
    escape_block::EscapeBlock,
    function::{Function, FunctionCall},
    identifier::{Ident, IdentParent},
    member_access::UnresolvedMemberAccess,
    structure::{StructAccess, StructInit},
    tuple::{Tuple, TupleAccess},
    type_signature::{
        BuiltinType, TypeEvalError, TypeSignature, TypeSignatureContext, TypeSignatureParent,
        TypeSignatureValue, Typed,
    },
    IrAlloc, NodeRef,
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

    pub fn unwrap_ident(self, ctx: &IrCtx<'a>) -> Ident<'a> {
        match &ctx[self] {
            Expr::Identifier(ident, _span) => *ident.clone(),
            _ => panic!("failed to unwrap expr as identifier"),
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
            Expr::Function(func) => func.get_span(ctx),
            Expr::FunctionCall(call) => call.get_span(ctx),
            Expr::Identifier(_, span) => Some(span),
            Expr::StructInit(st_init) => st_init.get_span(ctx),
            Expr::StructAccess(st_acc) => st_acc.get_span(ctx),
            Expr::TupleAccess(tup_acc) => tup_acc.get_span(ctx),
            Expr::EscapeBlock(esc_blk) => esc_blk.get_span(ctx),
            Expr::Assignment(asgn) => asgn.get_span(ctx),
            Expr::Tuple(tup) => tup.get_span(ctx),
            Expr::EnumInit(enm_init) => enm_init.get_span(ctx),
            Expr::UnresolvedMemberAccess(mem_acc) => Some(ctx[mem_acc].span.clone()),
        }
    }
}

impl<'a> IrLowerable<'a> for crate::ast::node::expression::Expr<'a> {
    type IrType = Expr<'a>;

    fn ir_lower(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self::IrType> {
        match self.value {
            crate::ast::node::expression::ExprValue::StringLiteral(str) => {
                Expr::StringLiteral(str, self.span).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::NumberLiteral(num) => {
                Expr::NumberLiteral(num, self.span).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::BoolLiteral(bool) => {
                Expr::BoolLiteral(bool, self.span).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::Function(func) => {
                Expr::Function(func.ir_lower(ctx)).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::FunctionCall(func_call) => {
                Expr::FunctionCall(func_call.ir_lower(ctx)).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::Identifier(id) => {
                let id_expr = Expr::Identifier(LateInit::empty(), Span::empty()).allocate(ctx);

                let span = id.span.clone();

                let unresolved_ident = ctx
                    .make_unresolved_ident(id, IdentParent::IdentExpr(id_expr).into())
                    .into();

                ctx[id_expr] = Expr::Identifier(unresolved_ident, span);

                id_expr
            }
            crate::ast::node::expression::ExprValue::StructInit(st_init) => {
                Expr::StructInit(st_init.ir_lower(ctx)).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::TupleAccess(tup_acc) => {
                Expr::TupleAccess(tup_acc.ir_lower(ctx)).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::EscapeBlock(esc) => {
                Expr::EscapeBlock(esc.ir_lower(ctx)).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::Assignment(asg) => Expr::Assignment(
                Assignment {
                    lhs: asg.lhs.ir_lower(ctx),
                    rhs: asg.rhs.ir_lower(ctx),
                    span: self.span,
                }
                .allocate(ctx),
            )
            .allocate(ctx),
            crate::ast::node::expression::ExprValue::Tuple(tup) => {
                Expr::Tuple(tup.ir_lower(ctx)).allocate(ctx)
            }
            crate::ast::node::expression::ExprValue::MemberAccess(mem_acc) => {
                Expr::UnresolvedMemberAccess(mem_acc.ir_lower(ctx)).allocate(ctx)
            }
        }
    }
}
