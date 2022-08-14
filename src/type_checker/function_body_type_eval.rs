use crate::{
    ir::{
        context::IrCtx,
        node::{
            expression::Expr,
            function::Function,
            statement::Stmt,
            type_signature::{BuiltinType, TypeEvalError, TypeSignature, Typed},
            NodeRef,
        },
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::coercion::coerce;

#[derive(Debug)]
pub enum FunctionTypeError<'a> {
    ExprValue(NodeRef<'a, Function<'a>>, Box<TypeEvalError<'a>>),
    ConflictingReturnTypes(
        NodeRef<'a, Function<'a>>,
        TypeSignature<'a>,
        TypeSignature<'a>,
    ),
    WrongNumberOfArgs {
        func: NodeRef<'a, Function<'a>>,
        expected: usize,
        actual: usize,
    },
}

pub fn eval_func_body_type_sig<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    func: NodeRef<'a, Function<'a>>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    let func_body = ctx[func].body;
    let body_type = stmt_type(ctx, symbols, func, func_body)?;

    if let Some(return_type) = ctx[func].return_type {
        if let Some(coerced_type) = coerce(body_type, return_type, ctx) {
            Ok(coerced_type)
        } else {
            Err(FunctionTypeError::ConflictingReturnTypes(
                func,
                return_type,
                body_type,
            ))
        }
    } else {
        Ok(body_type)
    }
}

fn expr_type<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    func: NodeRef<'a, Function<'a>>,
    expr: NodeRef<'a, Expr<'a>>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    expr.eval_type(symbols, ctx)
        .map_err(|err| FunctionTypeError::ExprValue(func, Box::new(err)))
}

fn stmt_type<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    func: NodeRef<'a, Function<'a>>,
    stmt: NodeRef<'a, Stmt<'a>>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    match ctx[stmt].clone() {
        Stmt::VariableDecl(_) => {
            symbols.visit_next_symbol(ctx);
            Ok(ctx.get_builtin_type_sig(BuiltinType::Void))
        }
        Stmt::FunctionDecl(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Void)),
        Stmt::StructDecl(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Void)),
        Stmt::EnumDecl(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Void)),
        Stmt::Expression(expr) => expr_type(ctx, symbols, func, expr),
        Stmt::Return(expr) => expr_type(ctx, symbols, func, expr),
        Stmt::Compound(stmts) => stmt_compound_type(ctx, symbols, func, stmts.clone()),
    }
}

fn stmt_compound_type<'a>(
    ctx: &mut IrCtx<'a>,
    symbols: &mut SymbolTableZipper<'a>,
    func: NodeRef<'a, Function<'a>>,
    stmts: Vec<NodeRef<'a, Stmt<'a>>>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    let mut type_sig: Option<TypeSignature<'a>> = None;

    for stmt in stmts {
        let stmt_type_sig = stmt_type(ctx, symbols, func, stmt)?;
        match &ctx[stmt] {
            Stmt::Compound(_) | Stmt::Return(_) => {
                if let Some(type_sig_val) = &type_sig {
                    if let Some(coerced_type) = coerce(*type_sig_val, stmt_type_sig, ctx) {
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

    Ok(type_sig.unwrap_or(ctx.get_builtin_type_sig(BuiltinType::Void)))
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ir::{
            node::type_signature::TypeSignatureValue,
            test_utils::utils::{lowered_ir, type_check},
        },
        type_checker::TypeCheckerError,
    };

    use super::*;

    #[test]
    fn test_func_call_wrong_arg_type() {
        let mut ir = lowered_ir("func f(a: Number) {}; f(true)").unwrap();
        match type_check(&mut ir) {
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig,
                expr_type,
            }) => {
                assert_eq!(
                    ir.ctx[type_sig],
                    TypeSignatureValue::Function {
                        args: vec![ir.ctx.get_builtin_type_sig(BuiltinType::Number)],
                        return_type: ir.ctx.get_builtin_type_sig(BuiltinType::Void)
                    }
                );

                assert_eq!(
                    ir.ctx[expr_type],
                    TypeSignatureValue::Function {
                        args: vec![ir.ctx.get_builtin_type_sig(BuiltinType::Boolean)],
                        return_type: ir.ctx.get_builtin_type_sig(BuiltinType::Void)
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_func_call_wrong_arg_amount() {
        let mut ir = lowered_ir("func f(a: Number) {}; f(2, 3)").unwrap();
        match type_check(&mut ir) {
            Err(TypeCheckerError::TypeSignatureMismatch {
                type_sig,
                expr_type,
            }) => {
                assert_eq!(
                    ir.ctx[type_sig],
                    TypeSignatureValue::Function {
                        args: vec![ir.ctx.get_builtin_type_sig(BuiltinType::Number)],
                        return_type: ir.ctx.get_builtin_type_sig(BuiltinType::Void)
                    }
                );

                assert_eq!(
                    ir.ctx[expr_type],
                    TypeSignatureValue::Function {
                        args: vec![
                            ir.ctx.get_builtin_type_sig(BuiltinType::Number),
                            ir.ctx.get_builtin_type_sig(BuiltinType::Number)
                        ],
                        return_type: ir.ctx.get_builtin_type_sig(BuiltinType::Void)
                    }
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_func_return_typecheck() {
        let mut ir = lowered_ir("func test() -> Number { return false }").unwrap();

        match type_check(&mut ir) {
            Err(TypeCheckerError::TypeEvalError(TypeEvalError::FunctionType(
                FunctionTypeError::ConflictingReturnTypes(_, a, b),
            ))) => {
                assert_eq!(a, ir.ctx.get_builtin_type_sig(BuiltinType::Number));
                assert_eq!(b, ir.ctx.get_builtin_type_sig(BuiltinType::Boolean));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_decl_inside_scope() {
        let mut ir = lowered_ir("let f = () -> Boolean { let a = true; return a }").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_))
    }

    #[test]
    fn test_func_type_deduce() {
        let mut ir = lowered_ir("let f: (Boolean) -> Boolean = (val) { return val }").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_))
    }
}
