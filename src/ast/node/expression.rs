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

impl<'a> Expr<'a> {
    pub fn value_type(&self) -> TypeSignature<'a> {
        match self {
            Expr::StringLiteral(_) => BuiltinType::String.into(),
            Expr::NumberLiteral(_) => BuiltinType::Number.into(),
            Expr::BoolLiteral(_) => BuiltinType::Bool.into(),
            Expr::Function(func) => TypeSignature::Function {
                args: Box::new(func.args.iter().map(|arg| arg.type_sig.clone()).collect()),
                return_type: Box::new(func.return_type.clone()),
            },
            Expr::FunctionCall(_) => todo!(),
            Expr::Identifier(_) => todo!(),
        }
    }
}
