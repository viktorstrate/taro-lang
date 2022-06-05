use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char as char_parser, digit1},
    combinator::{map, opt},
    error::context,
    sequence::{delimited, tuple},
};

use crate::ast::node::expression::Expr;

use super::{
    function::{function_call_expr, function_expr},
    identifier::identifier,
    structure::struct_init_expr,
    Res, Span,
};

pub fn expression(i: Span) -> Res<Span, Expr> {
    return context(
        "expression",
        alt((function_call_expr, non_fn_call_expression)),
    )(i);
}

pub fn non_fn_call_expression(i: Span) -> Res<Span, Expr> {
    return alt((
        struct_init_expr,
        expr_string_literal,
        expr_number_literal,
        expr_boolean_literal,
        expr_identifier,
        function_expr,
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
    context(
        "boolean",
        map(
            alt((map(tag("true"), |_| true), map(tag("false"), |_| false))),
            Expr::BoolLiteral,
        ),
    )(i)
}

pub fn expr_identifier(i: Span) -> Res<Span, Expr> {
    context("identifier expression", map(identifier, Expr::Identifier))(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::parser::new_span;

    use super::*;

    #[test]
    fn test_expr() {
        assert_matches!(
            expression(new_span("\"hello\"")),
            Ok((_, Expr::StringLiteral("hello")))
        );

        match expression(new_span("23")).unwrap().1 {
            Expr::NumberLiteral(val) => assert_eq!(val, 23.0),
            _ => assert!(false),
        }

        match expression(new_span("23.2")).unwrap().1 {
            Expr::NumberLiteral(val) => assert_eq!(val, 23.2),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_string_literal() {
        assert!(expr_string_literal(new_span("\"hello not closed")).is_err());
    }
}
