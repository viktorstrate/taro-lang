use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char as char_parser, digit1},
    sequence::delimited,
};

use crate::ast::Expr;

use super::Res;

pub fn expression(i: &str) -> Res<&str, Expr> {
    return alt((expr_string_literal, expr_number_literal))(i);
}

pub fn expr_string_literal(i: &str) -> Res<&str, Expr> {
    let string_value = take_until("\"");

    return delimited(char_parser('\"'), string_value, char_parser('\"'))(i)
        .map(|(i, str_val)| (i, Expr::StringLiteral(str_val)));
}

pub fn expr_number_literal(i: &str) -> Res<&str, Expr> {
    return digit1(i).map(|(i, digits)| {
        let num: f64 = digits.parse().expect("digits should be parsable as number");
        return (i, Expr::NumberLiteral(num));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr() {
        assert_eq!(
            expression("\"hello\""),
            Ok(("", Expr::StringLiteral("hello")))
        );

        assert_eq!(expression("23"), Ok(("", Expr::NumberLiteral(23.0))))
    }

    #[test]
    fn test_expr_string_literal() {
        assert!(expr_string_literal("\"hello not closed").is_err());
    }
}
