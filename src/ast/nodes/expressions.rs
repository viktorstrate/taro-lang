use super::type_signature::{BuiltinType, TypeSignature};

#[derive(PartialEq, Debug, Clone)]
pub enum Expr<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
}

impl<'a> Expr<'a> {
    pub fn value_type(&self) -> TypeSignature<'a> {
        match self {
            &Self::StringLiteral(..) => BuiltinType::String.into(),
            &Self::NumberLiteral(..) => BuiltinType::Number.into(),
            &Self::BoolLiteral(..) => BuiltinType::Bool.into(),
        }
    }
}
