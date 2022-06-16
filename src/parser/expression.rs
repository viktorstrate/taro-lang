use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char as char_parser, digit1},
    combinator::{map, opt},
    error::context,
    multi::{fold_many1, separated_list0},
    sequence::{delimited, preceded, tuple},
};

use crate::ast::node::{
    expression::Expr, function::FunctionCall, identifier::Ident, structure::StructAccess,
};

use super::{
    escape_block::escape_block, function::function_expr, identifier::identifier,
    structure::struct_init_expr, surround_brackets, token, BracketType, Res, Span,
};

pub fn expression(i: Span) -> Res<Span, Expr> {
    let (i, expr) = context(
        "expression",
        alt((
            struct_init_expr,
            expr_identifier,
            expr_string_literal,
            expr_number_literal,
            expr_boolean_literal,
            function_expr,
            map(escape_block, Expr::EscapeBlock),
        )),
    )(i)?;

    if let Ok(result) = expr_tail(&expr, i.clone()) {
        return Ok(result);
    } else {
        return Ok((i, expr));
    }
}

enum ExprTail<'a> {
    FuncCall(Vec<Expr<'a>>),
    StructAccess(Ident<'a>),
}

fn expr_tail<'a>(base: &Expr<'a>, i: Span<'a>) -> Res<Span<'a>, Expr<'a>> {
    fold_many1(
        alt((tail_func_call, tail_struct_access)),
        || base.clone(),
        |acc, expr_tail| match expr_tail {
            ExprTail::FuncCall(func_args) => Expr::FunctionCall(Box::new(FunctionCall {
                func: acc,
                params: func_args,
            })),
            ExprTail::StructAccess(attr_name) => Expr::StructAccess(StructAccess {
                struct_expr: Box::new(acc),
                attr_name,
            }),
        },
    )(i)
}

fn tail_func_call(i: Span) -> Res<Span, ExprTail> {
    let func_params = separated_list0(token(tag(",")), expression);

    map(
        surround_brackets(BracketType::Round, func_params),
        ExprTail::FuncCall,
    )(i)
}

fn tail_struct_access(i: Span) -> Res<Span, ExprTail> {
    map(
        preceded(token(tag(".")), identifier),
        ExprTail::StructAccess,
    )(i)
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

    use crate::{ast::node::identifier::Ident, parser::new_span};

    use super::*;

    #[test]
    fn test_expr_number() {
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
        assert_matches!(
            expression(new_span("\"hello\"")),
            Ok((_, Expr::StringLiteral("hello")))
        );

        assert!(expr_string_literal(new_span("\"hello not closed")).is_err());
    }

    #[test]
    fn test_expr_identifier() {
        match expression(new_span("ident_234")).unwrap().1 {
            Expr::Identifier(ident) => assert_eq!(ident, Ident::new_unplaced("ident_234")),
            _ => assert!(false),
        }

        match expression(new_span("$_ident")).unwrap().1 {
            Expr::Identifier(ident) => assert_eq!(ident, Ident::new_unplaced("$_ident")),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_tail() {
        let expr = expression(new_span("base()().first.next(123)")).unwrap().1;

        println!("{:#?}", expr);

        match expr {
            Expr::FunctionCall(call) => {
                assert_eq!(call.params.len(), 1);
            }
            _ => assert!(false),
        }
    }
}
