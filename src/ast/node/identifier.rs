use crate::{
    ast::ref_generator::RefID,
    parser::Span,
    symbols::{symbol_table::symbol_table_zipper::SymbolTableZipper, symbol_table::SymbolValue},
};
use std::{fmt::Debug, hash::Hash, io::Write};

pub trait Identifiable<'a> {
    fn name(&self) -> &Ident<'a>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdentValue<'a> {
    Named(&'a str),
    Anonymous(RefID),
}

#[derive(Clone)]
pub struct Ident<'a> {
    pub pos: Option<Span<'a>>,
    pub value: IdentValue<'a>,
}

impl<'a> Debug for Ident<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.pos {
            Some(pos) => f
                .debug_struct("Ident")
                .field("value", &self.value)
                .field("line", &pos.location_line())
                .field("column", &pos.get_column())
                .finish(),
            None => f.debug_struct("Ident").field("value", &self.value).finish(),
        }
    }
}

impl<'a> Ident<'a> {
    pub const fn new(pos: Span<'a>, value: &'a str) -> Self {
        Ident {
            pos: Some(pos),
            value: IdentValue::Named(value),
        }
    }

    pub const fn new_unplaced(value: &'a str) -> Self {
        Ident {
            pos: None,
            value: IdentValue::Named(value),
        }
    }

    pub const fn new_anon(ref_id: RefID) -> Self {
        Ident {
            pos: None,
            value: IdentValue::Anonymous(ref_id),
        }
    }
}

impl<'a> Ident<'a> {
    pub fn write<W: Write>(
        &self,
        writer: &mut W,
        symbols: &SymbolTableZipper<'a>,
    ) -> std::io::Result<()> {
        let symval = symbols.lookup(self).expect("identifier should exist");

        match self.value {
            IdentValue::Named(name) => match symval {
                SymbolValue::StructDecl(_) => {
                    writer.write_all(format!("struct_{}", &name).as_bytes())
                }
                _ => writer.write_all(name.as_bytes()),
            },
            IdentValue::Anonymous(ref_id) => match symval {
                SymbolValue::FuncDecl(_) => {
                    writer.write_all(format!("anon_func_{}", ref_id).as_bytes())
                }
                _ => unreachable!("only functions can be anonymous"),
            },
        }
    }
}

impl PartialEq for Ident<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Ident<'_> {}

impl Hash for Ident<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
