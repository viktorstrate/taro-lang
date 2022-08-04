use self::{
    assignment::Assignment,
    enumeration::EnumValue,
    escape_block::EscapeBlock,
    expression::Expr,
    function::{Function, FunctionArg, FunctionCall},
    statement::Stmt,
    structure::{StructAccess, StructAttr, StructInit, StructInitValue},
    tuple::{Tuple, TupleAccess},
};



pub mod assignment;
pub mod enumeration;
pub mod escape_block;
pub mod expression;
pub mod function;
pub mod identifier;
pub mod module;
pub mod statement;
pub mod structure;
pub mod tuple;
pub mod type_signature;

pub enum IrNode<'a, 'ctx> {
    Stmt(Stmt<'a, 'ctx>),
    Expr(Expr<'a, 'ctx>),
    FunctionArg(FunctionArg<'a, 'ctx>),
    StructAttr(StructAttr<'a, 'ctx>),
    EnumValue(EnumValue<'a, 'ctx>),
    Function(Function<'a, 'ctx>),
    FunctionCall(FunctionCall<'a, 'ctx>),
    StructInitValue(StructInitValue<'a, 'ctx>),
    StructInit(StructInit<'a, 'ctx>),
    StructAccess(StructAccess<'a, 'ctx>),
    TupleAccess(TupleAccess<'a, 'ctx>),
    Tuple(Tuple<'a, 'ctx>),
    Assignment(Assignment<'a, 'ctx>),
    EscapeBlock(EscapeBlock<'a, 'ctx>),
}
