use crate::ast::node::{identifier::Ident, type_signature::TypeSignature};

pub const BUILTIN_TYPES: &'static [Ident<'static>] = &[
    BuiltinType::String.ident(),
    BuiltinType::Number.ident(),
    BuiltinType::Bool.ident(),
    BuiltinType::Void.ident(),
];

#[derive(Debug)]
pub enum BuiltinType {
    String,
    Number,
    Bool,
    Void,
}

impl BuiltinType {
    pub const fn ident(&self) -> Ident<'static> {
        let value = match self {
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Bool => "Boolean",
            BuiltinType::Void => "Void",
        };

        Ident::new_unplaced(value)
    }

    pub const fn type_sig(&self) -> TypeSignature<'static> {
        TypeSignature::Base(self.ident())
    }
}
