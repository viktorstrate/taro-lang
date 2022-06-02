use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char as char_parser, digit1},
    sequence::delimited,
};

use crate::ast::Expr;

use super::{Res, Span};

pub fn expression(i: Span) -> Res<Span, Expr> {
    return alt((expr_string_literal, expr_number_literal))(i);
}

pub fn expr_string_literal(i: Span) -> Res<Span, Expr> {
    let string_value = take_until("\"");

    return delimited(char_parser('\"'), string_value, char_parser('\"'))(i)
        .map(|(i, str_val)| (i, Expr::StringLiteral(&str_val)));
}

pub fn expr_number_literal(i: Span) -> Res<Span, Expr> {
    return digit1(i).map(|(i, digits)| {
        let num: f64 = digits.parse().expect("digits should be parsable as number");
        return (i, Expr::NumberLiteral(num));
    });
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_expr() {
        assert_matches!(
            expression(Span::new("\"hello\"")),
            Ok((_, Expr::StringLiteral("hello")))
        );

        assert_matches!(
            expression(Span::new("23")),
            Ok((_, Expr::NumberLiteral(23.0)))
        )
    }

    #[test]
    fn test_expr_string_literal() {
        assert!(expr_string_literal(Span::new("\"hello not closed")).is_err());
    }
}
