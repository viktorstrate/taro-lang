use crate::{parser::Span};
use std::{fmt::Debug, hash::Hash};

pub trait Identifiable<'a, 'ctx> {
    fn name(&self) -> &'ctx Ident<'a>;
}

#[derive(Debug, Clone)]
pub enum Ident<'a> {
    Named { def_span: Span<'a>, name: &'a str },
    Anonymous,
}

impl Eq for &Ident<'_> {}

impl PartialEq for &Ident<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(*self, *other)
    }
}

impl Hash for &Ident<'_> {
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
