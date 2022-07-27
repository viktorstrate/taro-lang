use crate::{
    ast::node::{
        expression::Expr,
        function::Function,
        statement::Stmt,
        type_signature::{TypeEvalError, TypeSignature, Typed},
    },
    symbols::{builtin_types::BuiltinType, symbol_table::symbol_table_zipper::SymbolTableZipper},
};

#[derive(Debug)]
pub enum FunctionTypeError<'a> {
    ExprValue(Function<'a>, Box<TypeEvalError<'a>>),
    ConflictingReturnTypes(Function<'a>, TypeSignature<'a>, TypeSignature<'a>),
    WrongNumberOfArgs {
        func: Function<'a>,
        expected: usize,
        actual: usize,
    },
}

pub fn eval_func_body_type_sig<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    func: &Function<'a>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    let body_type = stmt_type(symbols, func, &func.body)?;

    if let Some(return_type) = &func.return_type {
        if let Some(coerced_type) = TypeSignature::coerce(&body_type, return_type) {
            Ok(coerced_type.clone())
        } else {
            Err(FunctionTypeError::ConflictingReturnTypes(
                func.clone(),
                return_type.clone(),
                body_type,
            ))
        }
    } else {
        Ok(body_type)
    }
}

fn expr_type<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    func: &Function<'a>,
    expr: &Expr<'a>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    expr.eval_type(symbols)
        .map_err(|err| FunctionTypeError::ExprValue(func.clone(), Box::new(err)))
}

fn stmt_type<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    func: &Function<'a>,
    stmt: &Stmt<'a>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    match stmt {
        Stmt::VariableDecl(_) => {
            symbols.visit_next_symbol();
            Ok(BuiltinType::Void.type_sig())
        }
        Stmt::FunctionDecl(_) => Ok(BuiltinType::Void.type_sig()),
        Stmt::StructDecl(_) => Ok(BuiltinType::Void.type_sig()),
        Stmt::Expression(expr) => expr_type(symbols, func, expr),
        Stmt::Return(expr) => expr_type(symbols, func, expr),
        Stmt::Compound(stmts) => stmt_compound_type(symbols, func, stmts),
    }
}

fn stmt_compound_type<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    func: &Function<'a>,
    stmts: &Vec<Stmt<'a>>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    let mut type_sig: Option<TypeSignature<'a>> = None;

    for stmt in stmts {
        let stmt_type_sig = stmt_type(symbols, func, stmt)?;
        match stmt {
            Stmt::Compound(_) | Stmt::Return(_) => {
                if let Some(type_sig_val) = &type_sig {
                    if let Some(coerced_type) = TypeSignature::coerce(type_sig_val, &stmt_type_sig)
                    {
                        // update type_sig to match new coerced value
                        type_sig = Some(coerced_type.clone())
                    } else {
                        return Err(FunctionTypeError::ConflictingReturnTypes(
                            func.clone(),
                            type_sig_val.clone(),
                            stmt_type_sig,
                        ));
                    }
                } else {
                    type_sig = Some(stmt_type_sig);
                }
            }
            _ => {}
        }
    }

    Ok(type_sig.unwrap_or(BuiltinType::Void.type_sig()))
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::test_utils::utils::type_check, parser::parse_ast, type_checker::TypeCheckerError,
    };

    use super::*;

    #[test]
    fn test_func_call_wrong_arg_type() {
        let mut ast = parse_ast("func f(a: Number) {}; f(true)").unwrap();
        match type_check(&mut ast) {
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig,
                expr_type,
            }) => {
                assert_eq!(
                    type_sig,
                    TypeSignature::Function {
                        args: vec![BuiltinType::Number.type_sig()],
                        return_type: Box::new(BuiltinType::Void.type_sig())
                    }
                );

                assert_eq!(
                    expr_type,
                    TypeSignature::Function {
                        args: vec![BuiltinType::Boolean.type_sig()],
                        return_type: Box::new(BuiltinType::Void.type_sig())
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_func_call_wrong_arg_amount() {
        let mut ast = parse_ast("func f(a: Number) {}; f(2, 3)").unwrap();
        match type_check(&mut ast) {
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig,
                expr_type,
            }) => {
                assert_eq!(
                    type_sig,
                    TypeSignature::Function {
                        args: vec![BuiltinType::Number.type_sig()],
                        return_type: Box::new(BuiltinType::Void.type_sig())
                    }
                );

                assert_eq!(
                    expr_type,
                    TypeSignature::Function {
                        args: vec![
                            BuiltinType::Number.type_sig(),
                            BuiltinType::Number.type_sig()
                        ],
                        return_type: Box::new(BuiltinType::Void.type_sig())
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_func_return_typecheck() {
        let mut ast = parse_ast("func test() -> Number { return false }").unwrap();

        match type_check(&mut ast) {
            Err(TypeCheckerError::TypeEvalError(TypeEvalError::FunctionType(
                FunctionTypeError::ConflictingReturnTypes(_, a, b),
            ))) => {
                assert_eq!(a, BuiltinType::Number.type_sig());
                assert_eq!(b, BuiltinType::Boolean.type_sig());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_decl_inside_scope() {
        let mut ast = parse_ast("let f = () -> Boolean { let a = true; return a }").unwrap();
        assert_matches!(type_check(&mut ast), Ok(_))
    }

    #[test]
    fn test_func_type_deduce() {
        let mut ast = parse_ast("let f: (Boolean) -> Boolean = (val) { return val }").unwrap();
        assert_matches!(type_check(&mut ast), Ok(_))
    }
}
