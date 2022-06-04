use super::{
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::TypeSignature,
};

#[derive(Debug, Clone)]
pub struct FunctionDecl<'a> {
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: TypeSignature<'a>,
    pub body: Box<Stmt<'a>>,
}

#[derive(Debug, Clone)]
pub struct FunctionExpr<'a> {
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: TypeSignature<'a>,
    pub body: Box<Stmt<'a>>,
}

#[derive(Debug, Clone)]
pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub type_sig: TypeSignature<'a>,
}

impl<'a> Identifiable<'a> for FunctionDecl<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

impl<'a> Identifiable<'a> for FunctionArg<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}
