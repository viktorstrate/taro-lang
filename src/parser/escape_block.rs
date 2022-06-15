use nom::{
    bytes::complete::{tag, take_till},
    combinator::{consumed, map, opt},
    error::context,
    multi::many0_count,
    sequence::{pair, preceded},
};

use crate::ast::node::escape_block::EscapeBlock;

use super::{surround_brackets, token, type_signature::type_signature, BracketType, Res, Span};

pub fn escape_block(i: Span) -> Res<Span, EscapeBlock> {
    // "@" [TYPE_SIG] "{" CONTENT "}"

    context(
        "escape block",
        map(
            pair(
                preceded(token(tag("@")), opt(type_signature)),
                surround_brackets(BracketType::Curly, escape_block_content),
            ),
            |(type_sig, content)| EscapeBlock {
                content: content.trim(),
                type_sig,
            },
        ),
    )(i)
}

pub fn escape_block_content<'a>(i: Span<'a>) -> Res<Span<'a>, &'a str> {
    // ( TEXT "{" CONTENT "}" * "}" )* TEXT "}"

    map(
        consumed(pair(
            many0_count(pair(
                take_till(|c| c == '{' || c == '}'),
                surround_brackets(BracketType::Curly, escape_block_content),
            )),
            take_till(|c| c == '}'),
        )),
        |(content, _)| content.as_ref(),
    )(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{parser::new_span, symbols::builtin_types::BuiltinType};

    use super::*;

    #[test]
    fn test_escape_block_brackets_balance() {
        assert_matches!(
            escape_block(new_span(
                "@{ const f = ({ a }) => { console.log({a}) }; f() }"
            ))
            .unwrap()
            .1,
            EscapeBlock {
                content: "const f = ({ a }) => { console.log({a}) }; f()",
                type_sig: None
            }
        )
    }

    #[test]
    fn test_typed_escape_block() {
        let block = escape_block(new_span("@Boolean{ true || false }"))
            .unwrap()
            .1;

        assert_eq!(block.content, "true || false");
        assert_eq!(block.type_sig, Some(BuiltinType::Boolean.type_sig()));
    }
}
