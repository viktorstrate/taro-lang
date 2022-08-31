use crate::parser::Span;
use std::{fmt::Debug, hash::Hash};

#[derive(Clone, Debug)]
pub struct Ident<'a> {
    pub span: Span<'a>,
    pub value: &'a str,
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
