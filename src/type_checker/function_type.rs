use crate::{
    ast::node::{
        expression::Expr,
        function::Function,
        statement::Stmt,
        type_signature::{TypeEvalError, TypeSignature, Typed},
    },
    symbols::{builtin_types::BuiltinType, symbol_table::symbol_table_zipper::SymbolTableZipper},
};

impl<'a> Typed<'a> for Function<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let args = self
            .args
            .iter()
            .map(|arg| arg.type_sig.clone())
            .collect::<Vec<_>>();

        symbols
            .enter_scope(self.name.clone())
            .expect("function should be located in current scope");

        let return_type = func_body_type_sig(symbols, self).map_err(TypeEvalError::FunctionType)?;

        symbols.exit_scope().unwrap();

        Ok(TypeSignature::Function {
            args,
            return_type: Box::new(return_type),
        })
    }
}

#[derive(Debug)]
pub enum FunctionTypeError<'a> {
    ExprValue(Box<TypeEvalError<'a>>),
    ConflictingReturnTypes(TypeSignature<'a>, TypeSignature<'a>),
}

fn func_body_type_sig<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    func: &Function<'a>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    stmt_type(symbols, &func.body)
}

fn expr_type<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    expr: &Expr<'a>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    expr.eval_type(symbols)
        .map_err(|err| FunctionTypeError::ExprValue(Box::new(err)))
}

fn stmt_type<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    stmt: &Stmt<'a>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    match stmt {
        Stmt::VariableDecl(_) => {
            symbols.visit_next_symbol();
            Ok(BuiltinType::Void.type_sig())
        }
        Stmt::FunctionDecl(_) => Ok(BuiltinType::Void.type_sig()),
        Stmt::StructDecl(_) => Ok(BuiltinType::Void.type_sig()),
        Stmt::Expression(expr) => expr_type(symbols, expr),
        Stmt::Return(expr) => expr_type(symbols, expr),
        Stmt::Compound(stmts) => stmt_compound_type(symbols, stmts),
    }
}

fn stmt_compound_type<'a>(
    symbols: &mut SymbolTableZipper<'a>,
    stmts: &Vec<Stmt<'a>>,
) -> Result<TypeSignature<'a>, FunctionTypeError<'a>> {
    let mut type_sig: Option<TypeSignature<'a>> = None;

    for stmt in stmts {
        let stmt_type_sig = stmt_type(symbols, stmt)?;
        match stmt {
            Stmt::Compound(_) | Stmt::Return(_) => {
                if let Some(type_sig_val) = &type_sig {
                    if let Some(coerced_type) = TypeSignature::coerce(type_sig_val, &stmt_type_sig)
                    {
                        // update type_sig to match new coerced value
                        type_sig = Some(coerced_type.clone())
                    } else {
                        return Err(FunctionTypeError::ConflictingReturnTypes(
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
