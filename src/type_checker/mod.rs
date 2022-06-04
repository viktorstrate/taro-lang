use crate::ast::node::type_signature::TypeSignature;

pub mod types_walker;

#[derive(Debug, PartialEq)]
pub enum TypeCheckerError<'a> {
    TypeSignatureMismatch {
        type_sig: TypeSignature<'a>,
        expr_type: TypeSignature<'a>,
    },
    CallNonFunction {
        ident_type: TypeSignature<'a>,
    },
}
