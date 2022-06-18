use crate::{ast::node::type_signature::TypeSignature, symbols::builtin_types::BuiltinType};

impl<'a> TypeSignature<'a> {
    pub fn can_coerce_to(&self, other: &Self) -> bool {
        if *self == BuiltinType::Untyped.type_sig() {
            true
        } else {
            *self == *other
        }
    }

    pub fn coerce<'b>(a: &'b Self, b: &'b Self) -> Option<&'b Self> {
        if a.can_coerce_to(b) {
            Some(a)
        } else if b.can_coerce_to(a) {
            Some(b)
        } else {
            None
        }
    }
}
