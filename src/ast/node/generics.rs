use crate::parser::Span;

use super::identifier::Ident;

#[derive(Debug, Clone)]
pub struct GenericsDecl<'a> {
    pub generics: Vec<Ident<'a>>,
    pub span: Span<'a>,
}
