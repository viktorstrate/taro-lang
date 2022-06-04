use crate::{
    ast::node::{
        expression::{Expr, ExprValueError},
        function::Function,
        statement::Stmt,
        type_signature::{BuiltinType, TypeSignature, Typed},
    },
    symbols::symbol_table_zipper::SymbolTableZipper,
};

#[derive(Debug)]
pub enum FunctionTypeError<'a> {
    ExprValue(Box<ExprValueError<'a>>),
    ConflictingReturnTypes,
}

pub fn func_body_type_sig<'a, Func: Function<'a>>(
    symbols: &SymbolTableZipper<'a>,
    func: &Func,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    stmt_type(symbols, func.body())
}

fn func_expr_type<'a>(
    symbols: &SymbolTableZipper<'a>,
    expr: &Expr<'a>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    expr.type_sig(symbols)
        .map_err(|err| FunctionTypeError::ExprValue(Box::new(err)))
}

fn stmt_type<'a>(
    symbols: &SymbolTableZipper<'a>,
    stmt: &Stmt<'a>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    match stmt {
        Stmt::VariableDecl(_) => Ok(BuiltinType::Void.into()),
        Stmt::FunctionDecl(_) => Ok(BuiltinType::Void.into()),
        Stmt::StructDecl(_) => Ok(BuiltinType::Void.into()),
        Stmt::Expression(expr) => func_expr_type(symbols, expr),
        Stmt::Return(expr) => func_expr_type(symbols, expr),
        Stmt::Compound(stmts) => stmt_compound_type(symbols, stmts),
    }
}

fn stmt_compound_type<'a>(
    symbols: &SymbolTableZipper<'a>,
    stmts: &Vec<Stmt<'a>>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    let mut type_sig: Option<TypeSignature<'a>> = None;

    for stmt in stmts {
        match stmt {
            Stmt::Compound(stmts) => {
                let compound_type = stmt_compound_type(symbols, stmts)?;
                if let Some(type_sig) = &type_sig {
                    if *type_sig != compound_type {
                        return Err(FunctionTypeError::ConflictingReturnTypes);
                    }
                } else {
                    type_sig = Some(compound_type);
                }
            }
            Stmt::Return(expr) => {
                let return_type = func_expr_type(symbols, expr)?;
                if let Some(type_sig) = &type_sig {
                    if *type_sig != return_type {
                        return Err(FunctionTypeError::ConflictingReturnTypes);
                    }
                } else {
                    type_sig = Some(return_type);
                }
            }
            // TODO: check if branches when implemented
            _ => {}
        }
    }

    Ok(type_sig.unwrap_or(BuiltinType::Void.into()))
}
