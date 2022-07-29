use std::{cell::RefCell, rc::Rc};

use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    error::context,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::ast::node::{
    expression::Expr,
    function::{Function, FunctionArg},
    identifier::Ident,
    statement::Stmt,
    type_signature::TypeSignature,
};

use super::{
    identifier::identifier, statement::statement, surround_brackets, token,
    type_signature::type_signature, ws, BracketType, Res, Span,
};

pub fn function_decl(i: Span) -> Res<Span, Stmt> {
    // func IDENT "(" FUNC_ARGS ")" [-> RETURN_SIG] "{" BODY "}"

    let (i, _) = token(tuple((tag("func"), ws)))(i)?;
    let (i, name) = context("function name", identifier)(i)?;

    let (i, args) = surround_brackets(BracketType::Round, function_args)(i)?;
    let (i, return_type) = return_signature(i)?;
    let (i, body) = context(
        "function body",
        surround_brackets(BracketType::Curly, statement),
    )(i)?;

    Ok((
        i,
        Stmt::FunctionDecl(Function {
            name,
            args,
            return_type,
            body: Box::new(body),
        }),
    ))
}

pub fn function_expr(i: Span) -> Res<Span, Expr> {
    // "(" FUNC_ARGS ")" [-> RETURN_SIG] "{" BODY "}"

    let (i, args) = surround_brackets(BracketType::Round, function_args)(i)?;
    let (i, return_type) = return_signature(i)?;
    let (mut i, body) = surround_brackets(BracketType::Curly, statement)(i)?;

    let name_ref = i.extra.ref_gen.make_ref();

    Ok((
        i,
        Expr::Function(Function {
            name: Ident::new_anon(name_ref),
            args,
            return_type,
            body: Box::new(body),
        }),
    ))
}

fn return_signature(i: Span) -> Res<Span, Option<TypeSignature>> {
    context(
        "return signature",
        opt(preceded(token(tag("->")), type_signature)),
    )(i)
}

fn function_args(i: Span) -> Res<Span, Vec<FunctionArg>> {
    separated_list0(token(tag(",")), function_arg)(i)
}

fn function_arg(i: Span) -> Res<Span, FunctionArg> {
    // IDENT [: TYPE_SIG]

    context(
        "function argument",
        map(
            tuple((
                identifier,
                context(
                    "argument type",
                    opt(preceded(token(tag(":")), type_signature)),
                ),
            )),
            |(name, type_sig)| FunctionArg {
                name,
                type_sig: Rc::new(RefCell::new(type_sig)),
            },
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::node::identifier::{Ident, IdentValue},
        parser::{expression::expression, new_span, parse_ast},
        symbols::builtin_types::BuiltinType,
    };

    use super::*;

    #[test]
    fn test_function_decl_minimal() {
        let func_stmt = function_decl(new_span("func f(){}")).unwrap().1;

        match func_stmt {
            Stmt::FunctionDecl(func) => {
                assert_eq!(func.name, Ident::new_unplaced("f"));
                assert_eq!(func.return_type, None);
                assert_eq!(func.args.len(), 0);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_function_decl() {
        let func_stmt = function_decl(new_span("func sum (a: Number, b: Number) -> Number {}"))
            .unwrap()
            .1;

        match func_stmt {
            Stmt::FunctionDecl(func) => {
                assert_eq!(func.name, Ident::new_unplaced("sum"));
                assert_eq!(func.return_type, Some(BuiltinType::Number.type_sig()));
                assert_eq!(func.args.len(), 2);
                assert_eq!(func.args[0].name, Ident::new_unplaced("a"));
                assert_eq!(func.args[1].name, Ident::new_unplaced("b"));
                assert_eq!(
                    *func.args[0].type_sig.borrow(),
                    Some(BuiltinType::Number.type_sig())
                );
                assert_eq!(
                    *func.args[1].type_sig.borrow(),
                    Some(BuiltinType::Number.type_sig())
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_function_expr() {
        let func_expr = function_expr(new_span("(a: Number, b: Number) {}"))
            .unwrap()
            .1;

        match func_expr {
            Expr::Function(func) => {
                assert_eq!(func.args.len(), 2);
                assert_eq!(func.args[0].name, Ident::new_unplaced("a"));
                assert_eq!(
                    *func.args[0].type_sig.borrow(),
                    Some(BuiltinType::Number.type_sig())
                );
                assert_eq!(func.args[1].name, Ident::new_unplaced("b"));
                assert_eq!(
                    *func.args[1].type_sig.borrow(),
                    Some(BuiltinType::Number.type_sig())
                );
                assert_eq!(func.return_type, None);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_function_var_assignment() {
        let ast = parse_ast("let f = (a: Number, b: Number) {}").unwrap();
        assert_eq!(ast.inner_module().stmts.len(), 1);
        let func_var_assignment = &ast.inner_module().stmts[0];

        match func_var_assignment {
            Stmt::VariableDecl(var_decl) => {
                assert_eq!(var_decl.name, Ident::new_unplaced("f"));
                match &var_decl.value {
                    Expr::Function(func) => {
                        assert_eq!(func.args.len(), 2);
                    }
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_function_call() {
        let func_call = expression(new_span("f(10, \"hello\")")).unwrap().1;

        match func_call {
            Expr::FunctionCall(func_call) => {
                assert_matches!(
                    func_call.func,
                    Expr::Identifier(Ident {
                        value: IdentValue::Named("f"),
                        pos: _
                    })
                );

                assert_eq!(func_call.params.len(), 2);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_double_function_call() {
        let expr = expression(new_span("f(10)(20)")).unwrap().1;

        match expr {
            Expr::FunctionCall(func_call_outer) => match func_call_outer.func {
                Expr::FunctionCall(func_call_inner) => {
                    assert_eq!(func_call_outer.params.len(), 1);
                    assert_eq!(func_call_inner.params.len(), 1);
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }
}
