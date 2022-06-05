use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::TypeSignature,
};

#[derive(Debug, Clone)]
pub struct Function<'a> {
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub body: Box<Stmt<'a>>,
}

#[derive(Debug, Clone)]
pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub type_sig: TypeSignature<'a>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall<'a> {
    pub func: Expr<'a>,
    pub params: Vec<Expr<'a>>,
}

impl<'a> Identifiable<'a> for Function<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

impl<'a> Identifiable<'a> for FunctionArg<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}
