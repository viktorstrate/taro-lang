use crate::ir::node::{
    identifier::Ident,
    type_signature::{TypeEvalError, TypeSignature},
};

use self::{
    assignment::AssignmentError, function_body_type_eval::FunctionTypeError,
    struct_type::StructTypeError,
};

pub mod assignment;
pub mod coercion;
pub mod function_body_type_eval;
pub mod struct_type;
pub mod type_inference;
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
    UntypedValue(),
    LookupError(Ident<'a>),
    AssignmentError(AssignmentError<'a>),
    StructError(StructTypeError<'a>),
}
