use nom::{
    character::complete::{alphanumeric0, multispace0, satisfy},
    combinator::{recognize, verify},
    sequence::{pair, preceded},
};

use crate::ast::node::identifier::Ident;

use super::{Res, Span};

const RESERVED_KEYWORDS: &'static [&str] =
    &["struct", "func", "return", "let", "mut", "true", "false"];

pub fn identifier(i: Span) -> Res<Span, Ident> {
    let ident_base = preceded(
        multispace0,
        recognize(pair(satisfy(|c| c.is_alphabetic()), alphanumeric0)),
    );

    let mut not_keyword_ident = verify(ident_base, |s: &Span| !RESERVED_KEYWORDS.contains(s));

    not_keyword_ident(i).map(|(i, val)| (i.clone(), Ident::new(i, &val)))
}
