use crate::type_checker::function_type::{func_body_type_sig, FunctionTypeError};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::{TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub struct FuncDecl<'a> {
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub body: Box<Stmt<'a>>,
}

#[derive(Debug, Clone)]
pub struct FunctionExpr<'a> {
    pub args: Vec<FunctionArg<'a>>,
    pub return_sig: Option<TypeSignature<'a>>,
    pub body: Box<Stmt<'a>>,
}

pub trait Function<'a> {
    fn args(&self) -> &Vec<FunctionArg<'a>>;
    fn return_type(&self) -> &Option<TypeSignature<'a>>;
    fn body(&self) -> &Stmt<'a>;
}

impl<'a> Function<'a> for FuncDecl<'a> {
    fn args(&self) -> &Vec<FunctionArg<'a>> {
        &self.args
    }

    fn return_type(&self) -> &Option<TypeSignature<'a>> {
        &self.return_type
    }

    fn body(&self) -> &Stmt<'a> {
        &self.body
    }
}

impl<'a> Function<'a> for FunctionExpr<'a> {
    fn args(&self) -> &Vec<FunctionArg<'a>> {
        &self.args
    }

    fn return_type(&self) -> &Option<TypeSignature<'a>> {
        &self.return_sig
    }

    fn body(&self) -> &Stmt<'a> {
        &self.body
    }
}

impl<'a, F> Typed<'a> for F
where
    F: Function<'a>,
{
    type Error = FunctionTypeError<'a>;

    fn type_sig(
        &self,
        symbols: &crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        let args = self
            .args()
            .iter()
            .map(|arg| arg.type_sig.clone())
            .collect::<Vec<_>>();

        let return_type = func_body_type_sig(symbols, self)?;

        Ok(TypeSignature::Function {
            args: Box::new(args),
            return_type: Box::new(return_type),
        })
    }
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

impl<'a> Identifiable<'a> for FuncDecl<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

impl<'a> Identifiable<'a> for FunctionArg<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}
