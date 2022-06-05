use super::{
    expression::{Expr, ExprValueError},
    function::Function,
    identifier::{Ident, Identifiable},
    structure::Struct,
    type_signature::{Mutability, TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    VariableDecl(VarDecl<'a>),
    FunctionDecl(Function<'a>),
    Compound(Vec<Stmt<'a>>),
    Expression(Expr<'a>),
    StructDecl(Struct<'a>),
    Return(Expr<'a>),
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

impl<'a> Typed<'a> for VarDecl<'a> {
    type Error = ExprValueError<'a>;

    fn type_sig(
        &self,
        _symbols: &mut crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        todo!()
    }
}
