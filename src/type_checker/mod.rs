use crate::ir::node::{
    enumeration::{EnumInit, EnumValue},
    function::FunctionCall,
    identifier::Ident,
    type_signature::{TypeEvalError, TypeSignature},
    NodeRef,
};

use self::{check_assignment::AssignmentError, check_struct::StructTypeError};

pub mod check_assignment;
pub mod check_enum;
pub mod check_struct;
pub mod coercion;
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
    UnknownEnumValue {
        enum_name: Ident<'a>,
        enum_value: Ident<'a>,
    },
    EnumInitArgCountMismatch(NodeRef<'a, EnumInit<'a>>, NodeRef<'a, EnumValue<'a>>),
}
