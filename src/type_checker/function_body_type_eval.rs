use id_arena::Id;

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
    match &ctx[stmt] {
        Stmt::VariableDecl(_) => {
            symbols.visit_next_symbol(ctx);
            Ok(ctx.get_builtin_type_sig(BuiltinType::Void))
        }
        Stmt::FunctionDecl(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Void)),
        Stmt::StructDecl(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Void)),
        Stmt::EnumDecl(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Void)),
        Stmt::Expression(expr) => expr_type(ctx, symbols, func, *expr),
        Stmt::Return(expr) => expr_type(ctx, symbols, func, *expr),
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

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use crate::{
//         ir::test_utils::utils::type_check, parser::parse_ast, type_checker::TypeCheckerError,
//     };

//     use super::*;

//     #[test]
//     fn test_func_call_wrong_arg_type() {
//         let mut ast = parse_ast("func f(a: Number) {}; f(true)").unwrap();
//         match type_check(&mut ast) {
//             Err(TypeCheckerError::TypeSignatureMismatch {
//                 type_sig,
//                 expr_type,
//             }) => {
//                 assert_eq!(
//                     type_sig,
//                     TypeSignature::Function {
//                         args: vec![BuiltinType::Number.type_sig()],
//                         return_type: Box::new(BuiltinType::Void.type_sig())
//                     }
//                 );

//                 assert_eq!(
//                     expr_type,
//                     TypeSignature::Function {
//                         args: vec![BuiltinType::Boolean.type_sig()],
//                         return_type: Box::new(BuiltinType::Void.type_sig())
//                     }
//                 );
//             }
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_func_call_wrong_arg_amount() {
//         let mut ast = parse_ast("func f(a: Number) {}; f(2, 3)").unwrap();
//         match type_check(&mut ast) {
//             Err(TypeCheckerError::TypeSignatureMismatch {
//                 type_sig,
//                 expr_type,
//             }) => {
//                 assert_eq!(
//                     type_sig,
//                     TypeSignature::Function {
//                         args: vec![BuiltinType::Number.type_sig()],
//                         return_type: Box::new(BuiltinType::Void.type_sig())
//                     }
//                 );

//                 assert_eq!(
//                     expr_type,
//                     TypeSignature::Function {
//                         args: vec![
//                             BuiltinType::Number.type_sig(),
//                             BuiltinType::Number.type_sig()
//                         ],
//                         return_type: Box::new(BuiltinType::Void.type_sig())
//                     }
//                 );
//             }
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_func_return_typecheck() {
//         let mut ast = parse_ast("func test() -> Number { return false }").unwrap();

//         match type_check(&mut ast) {
//             Err(TypeCheckerError::TypeEvalError(TypeEvalError::FunctionType(
//                 FunctionTypeError::ConflictingReturnTypes(_, a, b),
//             ))) => {
//                 assert_eq!(a, BuiltinType::Number.type_sig());
//                 assert_eq!(b, BuiltinType::Boolean.type_sig());
//             }
//             _ => assert!(false),
//         }
//     }

//     #[test]
//     fn test_decl_inside_scope() {
//         let mut ast = parse_ast("let f = () -> Boolean { let a = true; return a }").unwrap();
//         assert_matches!(type_check(&mut ast), Ok(_))
//     }

//     #[test]
//     fn test_func_type_deduce() {
//         let mut ast = parse_ast("let f: (Boolean) -> Boolean = (val) { return val }").unwrap();
//         assert_matches!(type_check(&mut ast), Ok(_))
//     }
// }
