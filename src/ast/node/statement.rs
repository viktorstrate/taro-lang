use super::{
    expression::Expr,
    function::FunctionDecl,
    identifier::{Ident, Identifiable},
    type_signature::{Mutability, TypeSignature},
};

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    VariableDecl(VarDecl<'a>),
    FunctionDecl(FunctionDecl<'a>),
    Compound(Vec<Stmt<'a>>),
}

#[derive(Debug, Clone)]
pub struct VarDecl<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub value: Expr<'a>,
}

impl<'a> Identifiable<'a> for VarDecl<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}
