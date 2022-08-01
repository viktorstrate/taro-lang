use std::fmt::Debug;

use crate::parser::Span;

use super::identifier::Ident;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeSignature<'a> {
    pub span: Span<'a>,
    pub value: TypeSignatureValue<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSignatureValue<'a> {
    Base(Ident<'a>),
    Function {
        args: Vec<TypeSignature<'a>>,
        return_type: Box<TypeSignature<'a>>,
    },
    Struct {
        name: Ident<'a>,
    },
    Enum {
        name: Ident<'a>,
    },
    Tuple(Vec<TypeSignature<'a>>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Mutability {
    Mutable,
    Immutable,
}

impl From<bool> for Mutability {
    fn from(val: bool) -> Self {
        if val {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }
    }
}

impl Into<bool> for Mutability {
    fn into(self) -> bool {
        self == Mutability::Mutable
    }
}
