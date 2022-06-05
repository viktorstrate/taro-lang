use crate::ast::node::{expression::ExprValueError, type_signature::TypeSignature};

use self::function_type::FunctionTypeError;

pub mod function_type;
pub mod specialize_type;
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
    ValueError(ExprValueError<'a>),
    FunctionError(FunctionTypeError<'a>),
}
