use id_arena::Id;

use crate::{ir::context::IrCtx, parser::Span};
use std::{fmt::Debug};

use super::type_signature::BuiltinType;

pub trait Identifiable<'a> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a>;
}

pub type Ident<'a> = Id<IdentValue<'a>>;

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

// impl Eq for &ResolvedIdentValue<'_> {}

// impl PartialEq for &ResolvedIdentValue<'_> {
//     fn eq(&self, other: &Self) -> bool {
//         std::ptr::eq(*self, *other)
//     }
// }

// impl Hash for &ResolvedIdentValue<'_> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         std::ptr::hash(*self, state)
//     }
// }

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
