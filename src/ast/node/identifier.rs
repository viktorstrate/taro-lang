use crate::parser::Span;
use std::{fmt::Debug, hash::Hash};

pub trait Identifiable<'a> {
    fn name(&self) -> &Ident<'a>;
}

#[derive(Clone)]
pub struct Ident<'a> {
    pub pos: Span<'a>,
    pub value: &'a str,
}

impl<'a> Debug for Ident<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ident")
            .field("line", &self.pos.location_line())
            .field("column", &self.pos.get_column())
            .field("value", &self.value)
            .finish()
    }
}

impl<'a> Ident<'a> {
    pub fn new(pos: Span<'a>, value: &'a str) -> Self {
        Ident {
            pos: pos,
            value: value,
        }
    }

    pub fn new_unplaced(value: &'a str) -> Self {
        Ident {
            pos: Span::new(""),
            value,
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
