use crate::symbols::{
    symbol_table::SymbolValue::{FuncDecl, VarDecl},
    symbol_table_zipper::SymbolTableZipper,
};

use super::{
    function::{FunctionCall, FunctionExpr},
    identifier::Ident,
    type_signature::{BuiltinType, TypeSignature},
};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Function(FunctionExpr<'a>),
    FunctionCall(Box<FunctionCall<'a>>),
    Identifier(Ident<'a>),
}

#[derive(Debug)]
pub enum ExprValueError<'a> {
    CallNonFunction(TypeSignature<'a>),
    UnknownIdentifier(Ident<'a>),
}

impl<'a> Expr<'a> {
    pub fn value_type(
        &self,
        symbols: &SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, ExprValueError<'a>> {
        match self {
            Expr::StringLiteral(_) => Ok(BuiltinType::String.into()),
            Expr::NumberLiteral(_) => Ok(BuiltinType::Number.into()),
            Expr::BoolLiteral(_) => Ok(BuiltinType::Bool.into()),
            Expr::Function(func) => Ok(TypeSignature::Function {
                args: Box::new(func.args.iter().map(|arg| arg.type_sig.clone()).collect()),
                return_type: Box::new(func.return_type.clone()),
            }),
            Expr::FunctionCall(call) => match call.func.value_type(symbols)? {
                TypeSignature::Function {
                    args: _,
                    return_type,
                } => Ok(*return_type),
                wrong_type => Err(ExprValueError::CallNonFunction(wrong_type)),
            },
            Expr::Identifier(ident) => {
                let sym_val = symbols
                    .locate(ident)
                    .ok_or(ExprValueError::UnknownIdentifier(ident.clone()))?;

                match sym_val {
                    VarDecl(var_decl) => var_decl.value.value_type(symbols),
                    FuncDecl(_) => todo!(),
                }
            }
        }
    }
}
