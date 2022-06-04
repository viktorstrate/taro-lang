use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char as char_parser, digit1},
    combinator::opt,
    sequence::{delimited, tuple},
};

use crate::ast::node::expression::Expr;

use super::{Res, Span};

pub fn expression(i: Span) -> Res<Span, Expr> {
    return alt((
        expr_string_literal,
        expr_number_literal,
        expr_boolean_literal,
    ))(i);
}

pub fn expr_string_literal(i: Span) -> Res<Span, Expr> {
    let string_value = take_until("\"");

    return delimited(char_parser('\"'), string_value, char_parser('\"'))(i)
        .map(|(i, str_val)| (i, Expr::StringLiteral(&str_val)));
}

pub fn expr_number_literal(i: Span) -> Res<Span, Expr> {
    let (i, num) = digit1(i)?;

    let (i, maybe_decimal) = opt(tuple((tag("."), digit1)))(i)?;
    let result: f64 = if let Some((_, decimal)) = maybe_decimal {
        format!("{num}.{decimal}").parse().unwrap()
    } else {
        num.parse().unwrap()
    };

    Ok((i, Expr::NumberLiteral(result)))
}

pub fn expr_boolean_literal(i: Span) -> Res<Span, Expr> {
    let (i, result) = alt((tag("true"), tag("false")))(i)?;
    Ok((i, Expr::BoolLiteral(*result == "true")))
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

        assert_eq!(
            expression(Span::new("23")).unwrap().1,
            Expr::NumberLiteral(23.0)
        );

        assert_eq!(
            expression(Span::new("23.2")).unwrap().1,
            Expr::NumberLiteral(23.2)
        );
    }

    #[test]
    fn test_expr_string_literal() {
        assert!(expr_string_literal(Span::new("\"hello not closed")).is_err());
    }
}