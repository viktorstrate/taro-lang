use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    error::context,
    multi::separated_list0,
    sequence::{pair, preceded, tuple},
};

use crate::ast::node::{
    expression::ExprValue,
    function::{Function, FunctionArg},
    identifier::Ident,
    type_signature::TypeSignature,
};

use super::{
    identifier::identifier, spaced, span, statement::statement, surround_brackets,
    type_signature::type_signature, ws, BracketType, Input, Res, Span,
};

pub fn function_decl(i: Input<'_>) -> Res<Input<'_>, Function<'_>> {
    // func IDENT "(" FUNC_ARGS ")" [-> RETURN_SIG] "{" BODY "}"

    map(
        pair(
            function_signature,
            context(
                "function body",
                surround_brackets(BracketType::Curly, statement),
            ),
        ),
        |((name, args, return_type, span), body)| Function {
            name: Some(name),
            args,
            return_type,
            body: Box::new(body),
            span,
        },
    )(i)
}

pub fn function_expr(i: Input<'_>) -> Res<Input<'_>, ExprValue<'_>> {
    // "(" FUNC_ARGS ")" [-> RETURN_SIG] "{" BODY "}"

    map(
        pair(
            span(pair(
                surround_brackets(BracketType::Round, function_args),
                return_signature,
            )),
            context(
                "function body",
                surround_brackets(BracketType::Curly, statement),
            ),
        ),
        |((span, (args, return_type)), body)| {
            ExprValue::Function(Function {
                name: None,
                args,
                return_type,
                body: Box::new(body),
                span,
            })
        },
    )(i)
}

pub fn function_signature(
    i: Input<'_>,
) -> Res<
    Input<'_>,
    (
        Ident<'_>,
        Vec<FunctionArg<'_>>,
        Option<TypeSignature<'_>>,
        Span<'_>,
    ),
> {
    // func IDENT "(" FUNC_ARGS ")" [-> RETURN_SIG]

    map(
        span(tuple((
            preceded(
                spaced(tuple((tag("func"), ws))),
                context("function name", identifier),
            ),
            surround_brackets(BracketType::Round, function_args),
            return_signature,
        ))),
        |(span, (name, args, return_type))| (name, args, return_type, span),
    )(i)
}

fn return_signature(i: Input<'_>) -> Res<Input<'_>, Option<TypeSignature<'_>>> {
    context(
        "return signature",
        opt(preceded(spaced(tag("->")), type_signature)),
    )(i)
}

fn function_args(i: Input<'_>) -> Res<Input<'_>, Vec<FunctionArg<'_>>> {
    separated_list0(spaced(tag(",")), function_arg)(i)
}

fn function_arg(i: Input<'_>) -> Res<Input<'_>, FunctionArg<'_>> {
    // IDENT [: TYPE_SIG]

    context(
        "function argument",
        map(
            span(tuple((
                identifier,
                context(
                    "argument type",
                    opt(preceded(spaced(tag(":")), type_signature)),
                ),
            ))),
            |(span, (name, type_sig))| FunctionArg {
                name,
                type_sig,
                span,
            },
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::{
            node::{expression::Expr, identifier::Ident},
            test_utils::{test_ident, test_type_sig},
        },
        ir::{
            node::{identifier::IdentKey, statement::Stmt},
            test_utils::utils::lowered_ir,
        },
        parser::{expression::expression, new_input},
    };

    use super::*;

    #[test]
    fn test_function_decl_minimal() {
        let func = function_decl(new_input("func f(){}")).unwrap().1;

        assert_eq!(func.name, Some(test_ident("f")));
        assert_eq!(func.return_type, None);
        assert_eq!(func.args.len(), 0);
    }

    #[test]
    fn test_function_decl() {
        let func = function_decl(new_input("func sum (a: Number, b: Number) -> Number {}"))
            .unwrap()
            .1;

        assert_eq!(func.name, Some(test_ident("sum")));
        assert_eq!(func.return_type, Some(test_type_sig("Number")));
        assert_eq!(func.args.len(), 2);
        assert_eq!(func.args[0].name, test_ident("a"));
        assert_eq!(func.args[1].name, test_ident("b"));
        assert_eq!(func.args[0].type_sig, Some(test_type_sig("Number")));
        assert_eq!(func.args[1].type_sig, Some(test_type_sig("Number")));
    }

    #[test]
    fn test_function_expr() {
        let func_expr = function_expr(new_input("(a: Number, b: Number) {}"))
            .unwrap()
            .1;

        match func_expr {
            ExprValue::Function(func) => {
                assert_eq!(func.args.len(), 2);
                assert_eq!(func.args[0].name, test_ident("a"));
                assert_eq!(func.args[0].type_sig, Some(test_type_sig("Number")));
                assert_eq!(func.args[1].name, test_ident("b"));
                assert_eq!(func.args[1].type_sig, Some(test_type_sig("Number")));
                assert_eq!(func.return_type, None);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_function_var_assignment() {
        let ir = lowered_ir("let f = (a: Number, b: Number) {}").unwrap();
        let stmts = ir.ctx[ir.ir.0.stmt_block].0.clone();
        assert_eq!(stmts.len(), 1);

        match ir.ctx[stmts[0]] {
            Stmt::VariableDecl(var_decl) => {
                assert_eq!(
                    IdentKey::from_ident(&ir.ctx, *ir.ctx[var_decl].name),
                    IdentKey::Named("f")
                );
                match ir.ctx[ir.ctx[var_decl].value] {
                    crate::ir::node::expression::Expr::Function(func) => {
                        assert_eq!(ir.ctx[func].args.len(), 2);
                    }
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_function_call() {
        let func_call = expression(new_input("f(10, \"hello\")")).unwrap().1.value;

        match func_call {
            ExprValue::FunctionCall(func_call) => {
                assert_matches!(
                    func_call.func,
                    Expr {
                        span: _,
                        value: ExprValue::Identifier(Ident {
                            span: _,
                            value: "f"
                        })
                    }
                );

                assert_eq!(func_call.args.len(), 2);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_double_function_call() {
        let expr = expression(new_input("f(10)(20)")).unwrap().1;

        match expr {
            Expr {
                span,
                value: ExprValue::FunctionCall(func_call_outer),
            } => {
                assert_eq!(span.fragment, "f(10)(20)");

                match func_call_outer.func {
                    Expr {
                        span,
                        value: ExprValue::FunctionCall(func_call_inner),
                    } => {
                        assert_eq!(span.fragment, "f(10)");
                        assert_eq!(func_call_outer.args.len(), 1);
                        assert_eq!(func_call_inner.args.len(), 1);
                    }
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }
}
