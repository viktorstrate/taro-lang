use crate::ir::node::{
    function::FunctionCall,
    identifier::Ident,
    type_signature::{TypeEvalError, TypeSignature},
    NodeRef,
};

use self::{assignment::AssignmentError, struct_type::StructTypeError};

pub mod assignment;
pub mod coercion;
pub mod struct_type;
pub mod type_inference;
pub mod type_resolver;
pub mod types_walker;

#[derive(Debug)]
pub enum TypeCheckerError<'a> {
    ConflictingTypes(TypeSignature<'a>, TypeSignature<'a>),
    UndeterminableTypes,
    TypeEval(TypeEvalError<'a>),
    LookupError(Ident<'a>),
    AssignmentError(AssignmentError<'a>),
    StructError(StructTypeError<'a>),
    FuncArgCountMismatch(TypeSignature<'a>, TypeSignature<'a>),
    FuncCallWrongArgAmount(NodeRef<'a, FunctionCall<'a>>),
}
