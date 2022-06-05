use crate::{
    symbols::{
        builtin_types::BuiltinType, symbol_table::SymbolValue,
        symbol_table_zipper::SymbolTableZipper,
    },
    type_checker::function_type::FunctionTypeError,
};

use super::{
    function::{Function, FunctionCall},
    identifier::Ident,
    structure::StructInit,
    type_signature::{TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Function(Function<'a>),
    FunctionCall(Box<FunctionCall<'a>>),
    Identifier(Ident<'a>),
    StructInit(StructInit<'a>),
}

#[derive(Debug)]
pub enum ExprValueError<'a> {
    CallNonFunction(TypeSignature<'a>),
    UnknownIdentifier(Ident<'a>),
    FunctionType(FunctionTypeError<'a>),
}

impl<'a> Typed<'a> for Expr<'a> {
    type Error = ExprValueError<'a>;

    fn type_sig(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, ExprValueError<'a>> {
        match self {
            Expr::StringLiteral(_) => Ok(BuiltinType::String.type_sig()),
            Expr::NumberLiteral(_) => Ok(BuiltinType::Number.type_sig()),
            Expr::BoolLiteral(_) => Ok(BuiltinType::Bool.type_sig()),
            Expr::Function(func) => func.type_sig(symbols).map_err(ExprValueError::FunctionType),
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
                    SymbolValue::BuiltinType(builtin) => Ok(TypeSignature::Base(builtin.clone())),
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
            Expr::StructInit(struct_init) => struct_init.type_sig(symbols),
        }
    }
}
