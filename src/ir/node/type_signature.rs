use std::fmt::Debug;

use super::{expression::Expr, identifier::Ident};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum TypeSignature<'a, 'ctx> {
    Builtin(BuiltinType),
    Function {
        args: Vec<&'ctx TypeSignature<'a, 'ctx>>,
        return_type: &'ctx TypeSignature<'a, 'ctx>,
    },
    Struct {
        name: &'ctx Ident<'a>,
    },
    Enum {
        name: &'ctx Ident<'a>,
    },
    Tuple(Vec<&'ctx TypeSignature<'a, 'ctx>>),
}

#[derive(Debug)]
pub enum TypeEvalError<'a, 'ctx> {
    Expression(Expr<'a, 'ctx>),
    // FunctionType(FunctionTypeError<'a>),
    CallNonFunction(TypeSignature<'a, 'ctx>),
    AccessNonStruct(TypeSignature<'a, 'ctx>),
    AccessNonTuple(TypeSignature<'a, 'ctx>),
    TupleAccessOutOfBounds {
        tuple_len: usize,
        access_item: usize,
    },
    UnknownIdentifier(Ident<'a>),
    UndeterminableType(Ident<'a>),
}

// #[allow(unused_variables)]
// pub trait Typed<'a>: Debug {
//     fn eval_type(
//         &self,
//         symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>>;

//     fn specified_type(&self) -> Option<TypeSignature<'a>> {
//         None
//     }

//     fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
//         Ok(())
//     }
// }

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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum BuiltinType {
    String,
    Number,
    Boolean,
    Void,
    Untyped,
}
