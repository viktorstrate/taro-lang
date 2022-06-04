use super::{
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::TypeSignature,
};

pub struct Function<'a> {
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: TypeSignature<'a>,
    pub body: Stmt<'a>,
}

pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub type_sig: TypeSignature<'a>,
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
