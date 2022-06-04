use nom::{
    bytes::complete::tag,
    combinator::opt,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::ast::node::{
    expression::Expr,
    function::{FunctionArg, FunctionCall, FunctionDecl, FunctionExpr},
    statement::Stmt,
    type_signature::BuiltinType,
};

use super::{
    expression::{expression, non_fn_call_expression},
    identifier::identifier,
    statement::{statement, type_signature},
    surround_brackets, token, ws, BracketType, Res, Span,
};

pub fn function_decl(i: Span) -> Res<Span, Stmt> {
    // func IDENT "(" FUNC_ARGS ")" [-> RETURN_SIG] "{" BODY "}"

    let (i, _) = token(tuple((tag("func"), ws)))(i)?;
    let (i, name) = identifier(i)?;

    let (i, args) = surround_brackets(BracketType::Round, function_args)(i)?;
    let (i, return_type) = opt(preceded(token(tag("->")), type_signature))(i)?;
    let (i, body) = surround_brackets(BracketType::Curly, statement)(i)?;

    Ok((
        i,
        Stmt::FunctionDecl(FunctionDecl {
            name,
            args,
            return_type: return_type.unwrap_or(BuiltinType::Void.into()),
            body: Box::new(body),
        }),
    ))
}

pub fn function_expr(i: Span) -> Res<Span, Expr> {
    // "(" FUNC_ARGS ")" [-> RETURN_SIG] "{" BODY "}"

    let (i, args) = surround_brackets(BracketType::Round, function_args)(i)?;
    let (i, return_type) = opt(preceded(token(tag("->")), type_signature))(i)?;
    let (i, body) = surround_brackets(BracketType::Curly, statement)(i)?;

    Ok((
        i,
        Expr::Function(FunctionExpr {
            args,
            return_type: return_type.unwrap_or(BuiltinType::Void.into()),
            body: Box::new(body),
        }),
    ))
}

fn function_args(i: Span) -> Res<Span, Vec<FunctionArg>> {
    separated_list0(token(tag(",")), function_arg)(i)
}

fn function_arg(i: Span) -> Res<Span, FunctionArg> {
    // IDENT : TYPE_SIG

    let (i, name) = identifier(i)?;
    let (i, _) = token(tag(":"))(i)?;
    let (i, type_sig) = type_signature(i)?;

    Ok((i, FunctionArg { name, type_sig }))
}

pub fn function_call_expr(i: Span) -> Res<Span, Expr> {
    // EXPR "(" FUNC_PARAMS ")"

    let func_params = separated_list0(token(tag(",")), expression);

    let (i, func) = non_fn_call_expression(i)?;
    let (i, params) = surround_brackets(BracketType::Round, func_params)(i)?;

    Ok((
        i,
        Expr::FunctionCall(Box::new(FunctionCall { func, params })),
    ))
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{ast::node::identifier::Ident, parser::parse_ast};

    use super::*;

    #[test]
    fn test_function_decl() {
        let func_stmt = function_decl(Span::new("func sum (a: Number, b: Number) -> Number {}"))
            .unwrap()
            .1;

        let Stmt::FunctionDecl(func) = func_stmt else {
            panic!();
        };

        assert_eq!(func.name.value, "sum");
        assert_eq!(func.return_type, BuiltinType::Number.into());
        assert_eq!(func.args.len(), 2);
        assert_eq!(func.args[0].name.value, "a");
        assert_eq!(func.args[1].name.value, "b");
        assert_eq!(func.args[0].type_sig, BuiltinType::Number.into());
        assert_eq!(func.args[1].type_sig, BuiltinType::Number.into());
    }

    #[test]
    fn test_function_expr() {
        let func_expr = function_expr(Span::new("(a: Number, b: Number) {}"))
            .unwrap()
            .1;

        match func_expr {
            Expr::Function(func) => {
                assert_eq!(func.args.len(), 2);
                assert_eq!(func.args[0].name.value, "a");
                assert_eq!(func.args[0].type_sig, BuiltinType::Number.into());
                assert_eq!(func.args[1].name.value, "b");
                assert_eq!(func.args[1].type_sig, BuiltinType::Number.into());
                assert_eq!(func.return_type, BuiltinType::Void.into())
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
                assert_eq!(var_decl.name.value, "f");
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
        let func_call = function_call_expr(Span::new("f(10, \"hello\")")).unwrap().1;

        match func_call {
            Expr::FunctionCall(func_call) => {
                assert_matches!(
                    func_call.func,
                    Expr::Identifier(Ident { value: "f", pos: _ })
                );

                assert_eq!(func_call.params.len(), 2);
            }
            _ => assert!(false),
        }
    }
}
