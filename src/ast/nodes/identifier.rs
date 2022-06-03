use crate::parser::Span;
use std::hash::Hash;

pub trait Identifiable<'a> {
    fn name(&self) -> &Ident<'a>;
}

#[derive(Debug, Clone)]
pub struct Ident<'a> {
    pub pos: Span<'a>,
    pub value: &'a str,
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
