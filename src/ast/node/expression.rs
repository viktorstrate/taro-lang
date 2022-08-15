use crate::parser::Span;

use super::{
    assignment::Assignment,
    enumeration::EnumInit,
    escape_block::EscapeBlock,
    function::{Function, FunctionCall},
    identifier::Ident,
    structure::{StructAccess, StructInit},
    tuple::{Tuple, TupleAccess},
};

#[derive(Debug, Clone)]
pub struct Expr<'a> {
    pub span: Span<'a>,
    pub value: ExprValue<'a>,
}

#[derive(Debug, Clone)]
pub enum ExprValue<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Function(Function<'a>),
    FunctionCall(Box<FunctionCall<'a>>),
    Identifier(Ident<'a>),
    StructInit(StructInit<'a>),
    StructAccess(StructAccess<'a>),
    TupleAccess(TupleAccess<'a>),
    EscapeBlock(EscapeBlock<'a>),
    Assignment(Box<Assignment<'a>>),
    Tuple(Tuple<'a>),
    EnumInit(EnumInit<'a>),
}
