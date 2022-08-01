use crate::parser::Span;

use super::node::{
    identifier::Ident,
    type_signature::{TypeSignature, TypeSignatureValue},
};

fn test_span() -> Span<'static> {
    Span {
        line: 0,
        offset: 0,
        fragment: "",
    }
}

pub fn test_ident<'a>(name: &'a str) -> Ident<'a> {
    Ident {
        span: test_span(),
        value: name,
    }
}

pub fn test_type_sig<'a>(name: &'a str) -> TypeSignature<'a> {
    TypeSignature {
        span: test_span(),
        value: TypeSignatureValue::Base(test_ident(name)),
    }
}
