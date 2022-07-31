use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char as char_parser, digit1},
    combinator::{map, opt},
    error::context,
    multi::{fold_many0, separated_list0},
    sequence::{delimited, preceded, tuple},
};

use crate::ast::node::{
    assignment::Assignment,
    expression::Expr,
    function::FunctionCall,
    identifier::Ident,
    structure::StructAccess,
    tuple::{Tuple, TupleAccess},
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
            expr_tuple,
            map(escape_block, Expr::EscapeBlock),
        )),
    )(i)?;

    expr_tail(&expr, i.clone())
}

enum ExprTailChain<'a> {
    FuncCall(Vec<Expr<'a>>),
    StructAccess(Ident<'a>),
    TupleAccess(usize),
}

fn expr_tail<'a>(base: &Expr<'a>, i: Span<'a>) -> Res<Span<'a>, Expr<'a>> {
    let (i, base) = expr_tail_chain(base.clone(), i)?;

    tail_assignments(&base, i)
}

fn tail_assignments<'a>(base: &Expr<'a>, i: Span<'a>) -> Res<Span<'a>, Expr<'a>> {
    fold_many0(
        preceded(token(tag("=")), expression),
        || base.clone(),
        |acc, rhs| Expr::Assignment(Box::new(Assignment { lhs: acc, rhs })),
    )(i)
}

fn expr_tail_chain<'a>(base: Expr<'a>, i: Span<'a>) -> Res<Span<'a>, Expr<'a>> {
    fold_many0(
        alt((tail_func_call, tail_struct_access, tail_tuple_access)),
        || base.clone(),
        |acc, expr_tail| match expr_tail {
            ExprTailChain::FuncCall(func_args) => Expr::FunctionCall(Box::new(FunctionCall {
                func: acc,
                params: func_args,
            })),
            ExprTailChain::StructAccess(attr_name) => Expr::StructAccess(StructAccess {
                struct_expr: Box::new(acc),
                attr_name,
            }),
            ExprTailChain::TupleAccess(attr) => Expr::TupleAccess(TupleAccess {
                tuple_expr: Box::new(acc),
                attr,
            }),
        },
    )(i)
}

fn tail_func_call(i: Span) -> Res<Span, ExprTailChain> {
    let func_params = separated_list0(token(tag(",")), expression);

    map(
        surround_brackets(BracketType::Round, func_params),
        ExprTailChain::FuncCall,
    )(i)
}

fn tail_struct_access(i: Span) -> Res<Span, ExprTailChain> {
    map(
        preceded(token(tag(".")), identifier),
        ExprTailChain::StructAccess,
    )(i)
}

fn tail_tuple_access(i: Span) -> Res<Span, ExprTailChain> {
    let (i, digit) = preceded(token(tag(".")), digit1)(i)?;
    let num = digit.parse().unwrap();

    Ok((i, ExprTailChain::TupleAccess(num)))
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

pub fn expr_tuple(i: Span) -> Res<Span, Expr> {
    context(
        "tuple expression",
        map(
            surround_brackets(
                BracketType::Round,
                separated_list0(token(tag(",")), expression),
            ),
            |exprs| {
                Expr::Tuple(Tuple {
                    values: exprs,
                    type_sig: None,
                })
            },
        ),
    )(i)
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
    fn test_expr_tail_chain() {
        let expr = expression(new_span("base()().first.next(123)")).unwrap().1;

        match expr {
            Expr::FunctionCall(call) => {
                assert_eq!(call.params.len(), 1);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_assignments() {
        let expr = expression(new_span("a = b = 2")).unwrap().1;

        match expr {
            Expr::Assignment(outer_asg) => {
                match outer_asg.lhs {
                    Expr::Identifier(ident) => assert_eq!(ident, Ident::new_unplaced("a")),
                    _ => assert!(false),
                }

                match outer_asg.rhs {
                    Expr::Assignment(rhs_asg) => {
                        assert_matches!(rhs_asg.lhs, Expr::Identifier(_));
                        assert_matches!(rhs_asg.rhs, Expr::NumberLiteral(_));
                    }
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_tuple() {
        let expr = expression(new_span("(true, 42)")).unwrap().1;

        match expr {
            Expr::Tuple(Tuple {
                values,
                type_sig: None,
            }) => {
                assert_eq!(values.len(), 2);
                assert_matches!(values[0], Expr::BoolLiteral(true));
            }
            _ => assert!(false),
        }
    }
}
