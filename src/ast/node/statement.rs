use super::{
    expression::Expr,
    function::Function,
    identifier::{Ident, Identifiable},
    structure::Struct,
    type_signature::{Mutability, TypeEvalError, TypeSignature, Typed},
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
    fn eval_type(
        &self,
        symbols: &mut crate::symbols::symbol_table::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        self.value.eval_type(symbols)
    }

    fn specified_type(&self) -> Option<TypeSignature<'a>> {
        self.type_sig.clone()
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) {
        self.type_sig = Some(new_type);
    }
}
