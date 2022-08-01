use std::{cell::RefCell, rc::Rc};

use super::{expression::Expr, identifier::Ident, statement::Stmt, type_signature::TypeSignature};

#[derive(Debug, Clone)]
pub struct Function<'a> {
    pub name: Option<Ident<'a>>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub body: Box<Stmt<'a>>,
}

#[derive(Debug, Clone)]
pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub type_sig: Rc<RefCell<Option<TypeSignature<'a>>>>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall<'a> {
    pub func: Expr<'a>,
    pub params: Vec<Expr<'a>>,
}
