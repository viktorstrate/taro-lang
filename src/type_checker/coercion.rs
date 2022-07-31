use crate::{ir::node::type_signature::TypeSignature, symbols::builtin_types::BuiltinType};

impl<'a> TypeSignature<'a> {
    pub fn can_coerce_to(&self, other: &Self) -> bool {
        if let (TypeSignature::Tuple(selves), TypeSignature::Tuple(others)) = (self, other) {
            selves
                .iter()
                .zip(others.iter())
                .all(|(slf, other)| slf.can_coerce_to(other))
        } else if *self == BuiltinType::Untyped.type_sig() {
            true
        } else {
            *self == *other
        }
    }

    pub fn coerce<'b>(a: &'b Self, b: &'b Self) -> Option<&'b Self> {
        if a.can_coerce_to(b) {
            Some(b)
        } else if b.can_coerce_to(a) {
            Some(a)
        } else {
            None
        }
    }
}
