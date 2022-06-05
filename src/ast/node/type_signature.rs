use crate::symbols::symbol_table_zipper::SymbolTableZipper;

use super::identifier::Ident;

#[derive(PartialEq, Debug, Clone)]
pub enum TypeSignature<'a> {
    Base(Ident<'a>),
    Function {
        args: Box<Vec<TypeSignature<'a>>>,
        return_type: Box<TypeSignature<'a>>,
    },
    Reference(Box<TypeSignature<'a>>),
    // GenericBase(Ident<'a>, Box<Vec<TypeSignatureValue<'a>>>),
}

pub trait Typed<'a> {
    type Error = ();

    fn type_sig(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, Self::Error>;
}

#[derive(Debug)]
pub enum BuiltinType {
    String,
    Number,
    Bool,
    Void,
}

impl Into<TypeSignature<'static>> for BuiltinType {
    fn into(self) -> TypeSignature<'static> {
        let value = match self {
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Bool => "Boolean",
            BuiltinType::Void => "Void",
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
