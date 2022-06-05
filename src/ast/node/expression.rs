use crate::{
    symbols::{symbol_table::SymbolValue, symbol_table_zipper::SymbolTableZipper},
    type_checker::function_type::FunctionTypeError,
};

use super::{
    function::{Function, FunctionCall},
    identifier::Ident,
    type_signature::{BuiltinType, TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Function(Function<'a>),
    FunctionCall(Box<FunctionCall<'a>>),
    Identifier(Ident<'a>),
}

#[derive(Debug)]
pub enum ExprValueError<'a> {
    CallNonFunction(TypeSignature<'a>),
    UnknownIdentifier(Ident<'a>),
    FunctionBodyType(FunctionTypeError<'a>),
}

impl<'a> Typed<'a> for Expr<'a> {
    type Error = ExprValueError<'a>;

    fn type_sig(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, ExprValueError<'a>> {
        match self {
            Expr::StringLiteral(_) => Ok(BuiltinType::String.into()),
            Expr::NumberLiteral(_) => Ok(BuiltinType::Number.into()),
            Expr::BoolLiteral(_) => Ok(BuiltinType::Bool.into()),
            Expr::Function(func) => func
                .type_sig(symbols)
                .map_err(ExprValueError::FunctionBodyType),
            Expr::FunctionCall(call) => match call.func.type_sig(symbols)? {
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
                    SymbolValue::VarDecl(var_decl) => var_decl.clone().value.type_sig(symbols),
                    SymbolValue::FuncDecl(func_decl) => Ok(func_decl
                        .clone()
                        .type_sig(symbols)
                        .expect("function type sig always succeeds")),
                    SymbolValue::FuncArg(arg) => Ok(arg.type_sig.clone()),
                    SymbolValue::StructDecl(st) => st.clone().type_sig(symbols),
                    SymbolValue::StructAttr(attr) => attr.clone().type_sig(symbols),
                }
            }
        }
    }
}
