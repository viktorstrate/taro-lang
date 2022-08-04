use std::{fmt::Debug, hash::Hash};

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
    // Struct {
    //     name: Ident<'a>,
    // },
    // Enum {
    //     name: Ident<'a>,
    // },
    Tuple(Vec<TypeSignature<'a>>),
}

impl Eq for TypeSignatureValue<'_> {}

impl<'a> Hash for TypeSignatureValue<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TypeSignatureValue::Base(base) => {
                state.write_u8(1);
                base.hash(state);
            }
            TypeSignatureValue::Function { args, return_type } => {
                state.write_u8(2);
                args.iter().for_each(|arg| arg.value.hash(state));
                return_type.value.hash(state);
            }
            TypeSignatureValue::Tuple(types) => {
                state.write_u8(3);
                types.iter().for_each(|t| t.value.hash(state));
            } // TypeSignatureValue::Struct { name } => {
              //     state.write_u8(4);
              //     name.hash(state);
              // }
              // TypeSignatureValue::Enum { name } => {
              //     state.write_u8(5);
              //     name.hash(state);
              // }
        }
    }
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
