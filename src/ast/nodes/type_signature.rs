use super::identifier::Ident;

#[derive(PartialEq, Debug, Clone)]
pub enum TypeSignature<'a> {
    Base(Ident<'a>),
    Function(
        Ident<'a>,
        Box<Vec<TypeSignature<'a>>>,
        Box<TypeSignature<'a>>,
    ),
    Reference(Box<TypeSignature<'a>>),
    // GenericBase(Ident<'a>, Box<Vec<TypeSignatureValue<'a>>>),
}

#[derive(Debug)]
pub enum BuiltinType {
    String,
    Number,
    Bool,
}

impl Into<TypeSignature<'static>> for BuiltinType {
    fn into(self) -> TypeSignature<'static> {
        let value = match self {
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Bool => "Bool",
        };
        TypeSignature::Base(Ident::new_unplaced(value))
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Mutability {
    Mutable,
    Immutable,
}

impl From<bool> for Mutability {
    fn from(val: bool) -> Self {
        if val {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }
    }
}

impl Into<bool> for Mutability {
    fn into(self) -> bool {
        self == Mutability::Mutable
    }
}
