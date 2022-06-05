use crate::{
    ast::ref_generator::RefID,
    type_checker::function_type::{func_body_type_sig, FunctionTypeError},
};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::{TypeSignature, Typed},
};

// #[derive(Debug, Clone)]
// pub enum FuncName<'a> {
//     Named(Ident<'a>),
//     Anonymous(RefID),
// }

// impl<'a> Identifiable<'a> for FuncName<'a> {
//     fn name(&self) -> &Ident<'a> {
//         match self {
//             FuncName::Named(ident) => ident,
//             FuncName::Anonymous(_) => todo!(),
//         }
//     }
// }

// impl<'a> From<Ident<'a>> for FuncName<'a> {
//     fn from(ident: Ident<'a>) -> Self {
//         Self::Named(ident)
//     }
// }

#[derive(Debug, Clone)]
pub struct Function<'a> {
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub body: Box<Stmt<'a>>,
}

impl<'a> Typed<'a> for Function<'a> {
    type Error = FunctionTypeError<'a>;

    fn type_sig(
        &self,
        symbols: &crate::symbols::symbol_table_zipper::SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error> {
        let args = self
            .args
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
