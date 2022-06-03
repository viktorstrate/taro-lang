use crate::ast::nodes::type_signature::TypeSignature;

pub mod types_walker;

#[derive(Debug, PartialEq)]
pub enum TypeCheckerError<'a> {
    TypeSignatureMismatch {
        type_sig: TypeSignature<'a>,
        expr_type: TypeSignature<'a>,
    },
}
