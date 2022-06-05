use crate::{
    symbols::symbol_table_zipper::SymbolTableZipper, type_checker::function_type::FunctionTypeError,
};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::{TypeSignature, Typed},
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

impl<'a> Typed<'a> for FunctionArg<'a> {
    type Error = FunctionTypeError<'a>;

    fn type_sig(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        Ok(self.type_sig.clone())
    }
}
