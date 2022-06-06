use crate::ast::node::type_signature::{TypeEvalError, TypeSignature, Typed};

use self::function_type::FunctionTypeError;

pub mod function_type;
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
}
