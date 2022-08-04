use crate::parser::Span;
use std::{cell::Cell, fmt::Debug, hash::Hash};

use super::type_signature::BuiltinType;

pub trait Identifiable<'a, 'ctx> {
    fn name(&self) -> &Ident<'a, 'ctx>;
}

#[derive(Debug)]
pub struct Ident<'a, 'ctx>(pub &'ctx Cell<&'ctx IdentValue<'a>>);

#[derive(Debug)]
pub enum IdentValue<'a> {
    Resolved(ResolvedIdentValue<'a>),
    Unresolved(crate::ast::node::identifier::Ident<'a>),
}

#[derive(Debug)]
pub enum ResolvedIdentValue<'a> {
    Named { def_span: Span<'a>, name: &'a str },
    Anonymous,
    BuiltinType(BuiltinType),
}

impl<'a, 'ctx> Copy for Ident<'a, 'ctx> {}

impl<'a, 'ctx> Clone for Ident<'a, 'ctx> {
    fn clone(&self) -> Self {
        Self((&self.0).clone())
    }
}

impl<'a, 'ctx> PartialEq for Ident<'a, 'ctx> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0.get(), other.0.get())
    }
}

impl<'a, 'ctx> Hash for Ident<'a, 'ctx> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0.get(), state)
    }
}

impl<'a, 'ctx> Eq for Ident<'a, 'ctx> {}

impl Eq for &ResolvedIdentValue<'_> {}

impl PartialEq for &ResolvedIdentValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(*self, *other)
    }
}

impl Hash for &ResolvedIdentValue<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(*self, state)
    }
}

// impl<'a> Ident<'a> {
//     pub fn write<W: Write>(
//         &self,
//         writer: &mut W,
//         symbols: &SymbolTableZipper<'a>,
//     ) -> std::io::Result<()> {
//         let symval = symbols.lookup(self).expect("identifier should exist");

//         match self.value {
//             IdentValue::Named(name) => match symval {
//                 _ => writer.write_all(name.as_bytes()),
//             },
//             IdentValue::Anonymous(ref_id) => match symval {
//                 SymbolValue::FuncDecl(_) => {
//                     writer.write_all(format!("anon_func_{}", ref_id).as_bytes())
//                 }
//                 _ => unreachable!("only functions can be anonymous"),
//             },
//         }
//     }
// }

// impl PartialEq for Ident<'_> {
//     fn eq(&self, other: &Self) -> bool {
//         self.value == other.value
//     }
// }

// impl Eq for Ident<'_> {}

// impl Hash for Ident<'_> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.value.hash(state);
//     }
// }
