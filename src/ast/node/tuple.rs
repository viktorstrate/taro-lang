use crate::parser::Span;

use super::{expression::Expr, type_signature::TypeSignature};

#[derive(Debug, Clone)]
pub struct Tuple<'a> {
    pub values: Vec<Expr<'a>>,
    pub type_sig: Option<TypeSignature<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct TupleAccess<'a> {
    pub tuple_expr: Box<Expr<'a>>,
    pub attr: usize,
}
