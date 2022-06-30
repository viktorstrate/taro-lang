use crate::ast::node::{
    identifier::Ident,
    type_signature::{TypeEvalError, TypeSignature, Typed},
};

use self::{
    assignment::AssignmentError, function_type::FunctionTypeError, struct_type::StructTypeError,
};

pub mod assignment;
pub mod coercion;
pub mod function_type;
pub mod struct_type;
pub mod types_helpers;
pub mod types_walker;

#[derive(Debug)]
pub enum TypeCheckerError<'a> {
    TypeSignatureMismatch {
        type_sig: TypeSignature<'a>,
        expr_type: TypeSignature<'a>,
    },
    CallNonFunction {
        ident_type: TypeSignature<'a>,
    },
    TypeEvalError(TypeEvalError<'a>),
    FunctionError(FunctionTypeError<'a>),
    UntypedValue(Box<dyn 'a + Typed<'a>>),
    LookupError(Ident<'a>),
    AssignmentError(AssignmentError<'a>),
    StructError(StructTypeError<'a>),
}
