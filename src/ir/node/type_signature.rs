use std::{cell::Cell, fmt::Debug};

use super::{expression::Expr, identifier::Ident};

#[derive(Debug)]
pub struct TypeSignature<'a, 'ctx>(pub &'ctx Cell<&'ctx TypeSignatureValue<'a, 'ctx>>);

impl<'a, 'ctx> Copy for TypeSignature<'a, 'ctx> {}

impl<'a, 'ctx> Clone for TypeSignature<'a, 'ctx> {
    fn clone(&self) -> Self {
        Self((&self.0).clone())
    }
}

impl<'a, 'ctx> PartialEq for TypeSignature<'a, 'ctx> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0.get(), other.0.get())
    }
}

#[derive(Debug)]
pub enum TypeSignatureValue<'a, 'ctx> {
    Builtin(BuiltinType),
    Unresolved(crate::ast::node::identifier::Ident<'a>),
    Function {
        args: Vec<TypeSignature<'a, 'ctx>>,
        return_type: TypeSignature<'a, 'ctx>,
    },
    Struct {
        name: Ident<'a, 'ctx>,
    },
    Enum {
        name: Ident<'a, 'ctx>,
    },
    Tuple(Vec<TypeSignature<'a, 'ctx>>),
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
    UnknownIdentifier(Ident<'a, 'ctx>),
    UndeterminableType(Ident<'a, 'ctx>),
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

pub type Mutability = crate::ast::node::type_signature::Mutability;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum BuiltinType {
    String,
    Number,
    Boolean,
    Void,
    Untyped,
}

pub const BUILTIN_TYPES: &'static [BuiltinType] = &[
    BuiltinType::String,
    BuiltinType::Number,
    BuiltinType::Boolean,
    BuiltinType::Void,
    BuiltinType::Untyped,
];

impl BuiltinType {
    pub const fn name(&self) -> &'static str {
        match self {
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Boolean => "Boolean",
            BuiltinType::Void => "Void",
            BuiltinType::Untyped => "Untyped",
        }
    }
}
