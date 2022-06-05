use crate::{ast::ref_generator::RefID, symbols::symbol_table_zipper::SymbolTableZipper};

use super::identifier::Ident;

#[derive(PartialEq, Debug, Clone)]
pub enum TypeSignature<'a> {
    Base(Ident<'a>),
    Function {
        args: Vec<TypeSignature<'a>>,
        return_type: Box<TypeSignature<'a>>,
    },
    Struct {
        name: Ident<'a>,
        ref_id: RefID,
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
