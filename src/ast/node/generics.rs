use crate::parser::Span;

use super::identifier::Ident;

#[derive(Debug, Clone)]
pub struct GenericsDecl<'a> {
    pub generics: Vec<GenericType<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct GenericType<'a> {
    pub name: Ident<'a>,
    pub span: Span<'a>,
}
