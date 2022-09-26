use nom::{
    character::complete::{satisfy},
    combinator::{map, recognize, verify},
    error::context,
    multi::many0,
    sequence::{pair},
};

use crate::ast::node::identifier::Ident;

use super::{spaced, span, Input, Res};

const RESERVED_KEYWORDS: &'static [&str] =
    &["struct", "func", "return", "let", "var", "true", "false"];

pub fn identifier(i: Input<'_>) -> Res<Input<'_>, Ident<'_>> {
    let ident_base = recognize(pair(
        satisfy(|c| c.is_alphabetic() || ['_', '$'].contains(&c)),
        many0(satisfy(|c| c.is_alphanumeric() || ['_', '$'].contains(&c))),
    ));

    let not_keyword_ident = context(
        "identifier",
        span(verify(ident_base, |s: &Input<'_>| {
            !RESERVED_KEYWORDS.contains(s)
        })),
    );

    map(spaced(not_keyword_ident), |(span, val)| Ident {
        span,
        value: *val,
    })(i)
}

#[cfg(test)]
mod tests {
    use crate::parser::new_input;

    use super::*;

    #[test]
    fn test_valid_identifier() {
        assert_eq!(
            identifier(new_input("hello123")).unwrap().1.value,
            "hello123"
        );
    }
}
