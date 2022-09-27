use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char as char_parser, digit1},
    combinator::{map, opt, recognize},
    error::context,
    multi::{fold_many0, separated_list0},
    sequence::{delimited, pair, preceded},
};
use nom_locate::position;

use crate::ast::node::{
    assignment::Assignment,
    expression::{Expr, ExprValue},
    function::FunctionCall,
    identifier::Ident,
    member_access::MemberAccess,
    tuple::{Tuple, TupleAccess},
};

use super::{
    decimal, escape_block::escape_block, function::function_expr, identifier::identifier, spaced,
    span, structure::struct_init_expr, surround_brackets, BracketType, Input, Res, Span,
};

pub fn expression(i: Input<'_>) -> Res<Input<'_>, Expr<'_>> {
    let (i_next, expr) = context(
        "expression",
        map(
            span(alt((
                map(struct_init_expr, ExprValue::StructInit),
                map(identifier, ExprValue::Identifier),
                expr_anon_member_access,
                expr_string_literal,
                expr_number_literal,
                expr_boolean_literal,
                function_expr,
                expr_tuple,
                map(escape_block, ExprValue::EscapeBlock),
            ))),
            |(span, value)| Expr { span, value },
        ),
    )(i.clone())?;

    expr_tail(&expr, i_next, i)
}

enum ExprTailChain<'a> {
    FuncCall {
        args: Vec<Expr<'a>>,
        args_span: Span<'a>,
    },
    MemberAccess(Ident<'a>, Option<(Span<'a>, Vec<Expr<'a>>)>),
    TupleAccess(usize),
}

fn expr_tail<'a>(base: &Expr<'a>, i: Input<'a>, i_start: Input<'a>) -> Res<Input<'a>, Expr<'a>> {
    let (i_next, base) = expr_tail_chain(base.clone(), i, i_start.clone())?;

    tail_assignments(&base, i_next, i_start)
}

fn tail_assignments<'a>(
    base: &Expr<'a>,
    i: Input<'a>,
    i_start: Input<'a>,
) -> Res<Input<'a>, Expr<'a>> {
    fold_many0(
        pair(preceded(spaced(tag("=")), expression), position),
        || base.clone(),
        |acc, (rhs, end)| Expr {
            span: Span::new(i_start.clone(), end),
            value: ExprValue::Assignment(Box::new(Assignment { lhs: acc, rhs })),
        },
    )(i)
}

fn expr_tail_chain<'a>(
    base: Expr<'a>,
    i: Input<'a>,
    i_start: Input<'a>,
) -> Res<Input<'a>, Expr<'a>> {
    fold_many0(
        pair(
            alt((tail_func_call, tail_member_access, tail_tuple_access)),
            position,
        ),
        || base.clone(),
        |acc, (expr_tail, end)| {
            let expr_val = match expr_tail {
                ExprTailChain::FuncCall { args, args_span } => {
                    ExprValue::FunctionCall(Box::new(FunctionCall {
                        func: acc,
                        args,
                        args_span,
                    }))
                }
                ExprTailChain::MemberAccess(member_name, items) => {
                    ExprValue::MemberAccess(Box::new(MemberAccess {
                        object: Some(acc),
                        member_name,
                        items,
                    }))
                }
                ExprTailChain::TupleAccess(attr) => ExprValue::TupleAccess(TupleAccess {
                    tuple_expr: Box::new(acc),
                    attr,
                }),
            };

            Expr {
                span: Span::new(i_start.clone(), end),
                value: expr_val,
            }
        },
    )(i)
}

pub fn expr_args(i: Input<'_>) -> Res<Input<'_>, Vec<Expr<'_>>> {
    // "(" EXPR+ ")"
    surround_brackets(
        BracketType::Round,
        separated_list0(spaced(tag(",")), expression),
    )(i)
}

fn tail_func_call(i: Input<'_>) -> Res<Input<'_>, ExprTailChain<'_>> {
    map(span(expr_args), |(args_span, args)| {
        ExprTailChain::FuncCall { args, args_span }
    })(i)
}

fn tail_member_access(i: Input<'_>) -> Res<Input<'_>, ExprTailChain<'_>> {
    map(
        pair(preceded(spaced(tag(".")), identifier), opt(span(expr_args))),
        |(member_name, items)| ExprTailChain::MemberAccess(member_name, items),
    )(i)
}

fn tail_tuple_access(i: Input<'_>) -> Res<Input<'_>, ExprTailChain<'_>> {
    let (i, digit) = preceded(spaced(tag(".")), digit1)(i)?;
    let num = digit.parse().unwrap();

    Ok((i, ExprTailChain::TupleAccess(num)))
}

pub fn expr_string_literal(i: Input<'_>) -> Res<Input<'_>, ExprValue<'_>> {
    let string_value = take_until("\"");

    return delimited(char_parser('\"'), string_value, char_parser('\"'))(i)
        .map(|(i, str_val)| (i, ExprValue::StringLiteral(&str_val)));
}

pub fn expr_number_literal(i: Input<'_>) -> Res<Input<'_>, ExprValue<'_>> {
    map(
        recognize(pair(decimal, opt(pair(char_parser('.'), opt(decimal))))),
        |num| ExprValue::NumberLiteral(num.parse().unwrap()),
    )(i)
}

pub fn expr_boolean_literal(i: Input<'_>) -> Res<Input<'_>, ExprValue<'_>> {
    context(
        "boolean",
        map(
            alt((map(tag("true"), |_| true), map(tag("false"), |_| false))),
            ExprValue::BoolLiteral,
        ),
    )(i)
}

pub fn expr_tuple(i: Input<'_>) -> Res<Input<'_>, ExprValue<'_>> {
    context(
        "tuple expression",
        map(expr_args, |exprs| {
            ExprValue::Tuple(Tuple {
                values: exprs,
                type_sig: None,
            })
        }),
    )(i)
}

pub fn expr_anon_member_access(i: Input<'_>) -> Res<Input<'_>, ExprValue<'_>> {
    // "." IDENT [ "(" EXPR+ ")" ]

    map(
        pair(preceded(spaced(tag(".")), identifier), opt(span(expr_args))),
        |(member_name, items)| {
            ExprValue::MemberAccess(Box::new(MemberAccess {
                object: None,
                member_name,
                items,
            }))
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::test_utils::test_ident,
        parser::{new_input, Span},
    };

    use super::*;

    #[test]
    fn test_expr_number() {
        match expression(new_input("23")).unwrap().1 {
            Expr {
                span,
                value: ExprValue::NumberLiteral(val),
            } => {
                assert_eq!(val, 23.0);
                assert_eq!(span.fragment, "23")
            }
            _ => assert!(false),
        }

        match expression(new_input("23.2")).unwrap().1.value {
            ExprValue::NumberLiteral(val) => assert_eq!(val, 23.2),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_string_literal() {
        assert_matches!(
            expression(new_input("\"hello\"")),
            Ok((
                _,
                Expr {
                    span: Span {
                        line: 1,
                        offset: 1,
                        fragment: "\"hello\"",
                        source: _
                    },
                    value: ExprValue::StringLiteral("hello")
                }
            ))
        );

        assert!(expr_string_literal(new_input("\"hello not closed")).is_err());
    }

    #[test]
    fn test_expr_identifier() {
        match expression(new_input("ident_234")).unwrap().1 {
            Expr {
                span,
                value: ExprValue::Identifier(ident),
            } => {
                assert_eq!(ident, test_ident("ident_234"));
                assert_eq!(span.fragment, "ident_234");
            }
            _ => assert!(false),
        }

        match expression(new_input("$_ident")).unwrap().1 {
            Expr {
                span,
                value: ExprValue::Identifier(ident),
            } => {
                assert_eq!(ident, test_ident("$_ident"));
                assert_eq!(span.fragment, "$_ident");
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_tail_chain() {
        let expr = expression(new_input("base()().first.next(123)")).unwrap().1;

        match expr {
            Expr {
                span,
                value: ExprValue::MemberAccess(mem_acc),
            } => match *mem_acc {
                MemberAccess {
                    object: _,
                    member_name,
                    items,
                } => {
                    assert_eq!(items.unwrap().1.len(), 1);
                    assert_eq!(member_name, test_ident("next"));
                    assert_eq!(span.fragment, "base()().first.next(123)");
                }
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_assignments() {
        let expr = expression(new_input("a = b = 2")).unwrap().1;

        match expr {
            Expr {
                span,
                value: ExprValue::Assignment(outer_asg),
            } => {
                assert_eq!(span.fragment, "a = b = 2");

                match outer_asg.lhs.value {
                    ExprValue::Identifier(ident) => assert_eq!(ident, test_ident("a")),
                    _ => assert!(false),
                }

                match outer_asg.rhs.value {
                    ExprValue::Assignment(rhs_asg) => {
                        assert_matches!(
                            rhs_asg.lhs,
                            Expr {
                                span: _,
                                value: ExprValue::Identifier(_)
                            }
                        );
                        assert_matches!(
                            rhs_asg.rhs,
                            Expr {
                                span: _,
                                value: ExprValue::NumberLiteral(_)
                            }
                        );
                    }
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_tuple() {
        let expr = expression(new_input("(true, 42)")).unwrap().1;

        match expr {
            Expr {
                span,
                value:
                    ExprValue::Tuple(Tuple {
                        values,
                        type_sig: None,
                    }),
            } => {
                assert_eq!(values.len(), 2);
                assert_matches!(values[0].value, ExprValue::BoolLiteral(true));

                assert_eq!(span.fragment, "(true, 42)");
                assert_eq!(values[0].span.fragment, "true");
                assert_eq!(values[1].span.fragment, "42");
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_expr_enum_init() {
        let expr = expression(new_input("IPAddress.v4(192, 168, 0, 1)"))
            .unwrap()
            .1;
        match expr {
            Expr {
                span: _,
                value: ExprValue::MemberAccess(mem_acc),
            } => {
                match mem_acc.object {
                    Some(Expr {
                        span: _,
                        value: ExprValue::Identifier(enum_name),
                    }) => {
                        assert_eq!(enum_name.value, "IPAddress");
                    }
                    _ => assert!(false),
                };

                assert_eq!(mem_acc.member_name, test_ident("v4"));
                assert_matches!(mem_acc.items.unwrap().1.len(), 4);
            }
            expr => assert!(false, "Expected EnumInit expression, got {expr:?}"),
        }
    }
}
