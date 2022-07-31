use super::{
    assignment::Assignment,
    escape_block::EscapeBlock,
    function::{Function, FunctionCall},
    identifier::Ident,
    structure::{StructAccess, StructInit},
    tuple::{Tuple, TupleAccess},
};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
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
}
