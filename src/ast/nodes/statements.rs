use super::{
    expressions::Expr,
    identifier::{Ident, Identifiable},
    type_signature::{Mutability, TypeSignature},
};

#[derive(PartialEq, Debug)]
pub enum Stmt<'a> {
    VarDecl(VarDecl<'a>),
    Compound(Vec<Stmt<'a>>),
}

#[derive(PartialEq, Debug, Clone)]
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
