use crate::{ast::ref_generator::RefID, parser::Span};
use std::{fmt::Debug, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdentValue<'a> {
    Named(&'a str),
    Anonymous(RefID),
}

#[derive(Clone)]
pub struct Ident<'a> {
    pub span: Option<Span<'a>>,
    pub value: IdentValue<'a>,
}

impl<'a> Debug for Ident<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.span {
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
            span: Some(pos),
            value: IdentValue::Named(value),
        }
    }

    pub const fn new_unplaced(value: &'a str) -> Self {
        Ident {
            span: None,
            value: IdentValue::Named(value),
        }
    }

    pub const fn new_anon(ref_id: RefID) -> Self {
        Ident {
            span: None,
            value: IdentValue::Anonymous(ref_id),
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
