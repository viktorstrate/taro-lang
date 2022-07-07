use std::{cell::RefCell, rc::Rc};

use crate::{
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
    type_checker::function_body_type_eval::eval_func_body_type_sig,
};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::{TypeEvalError, TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub struct Function<'a> {
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub body: Box<Stmt<'a>>,
}

impl<'a> Function<'a> {
    pub fn calculate_type_sig(
        args: &Vec<FunctionArg<'a>>,
        return_type: &Option<TypeSignature<'a>>,
    ) -> Option<TypeSignature<'a>> {
        let arg_types = args
            .iter()
            .map(|arg| arg.type_sig.borrow().clone())
            .collect::<Option<Vec<_>>>();

        match (arg_types, return_type) {
            (Some(args), Some(return_type)) => Some(TypeSignature::Function {
                args,
                return_type: Box::new(return_type.clone()),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub type_sig: Rc<RefCell<Option<TypeSignature<'a>>>>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall<'a> {
    pub func: Expr<'a>,
    pub params: Vec<Expr<'a>>,
}

impl<'a> Identifiable<'a> for Function<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

impl<'a> Identifiable<'a> for FunctionArg<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

impl<'a> Typed<'a> for Function<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let args = self
            .args
            .iter()
            .map(|arg| arg.eval_type(symbols))
            .collect::<Result<Vec<_>, _>>()?;

        symbols
            .enter_scope(self.name.clone())
            .expect("function should be located in current scope");

        let return_type =
            eval_func_body_type_sig(symbols, self).map_err(TypeEvalError::FunctionType)?;

        symbols.exit_scope().unwrap();

        Ok(TypeSignature::Function {
            args,
            return_type: Box::new(return_type),
        })
    }

    fn specified_type(&self) -> Option<TypeSignature<'a>> {
        Self::calculate_type_sig(&self.args, &self.return_type)
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) {
        let TypeSignature::Function { args, return_type: _ } = &new_type else {
            unreachable!("specified type expected to be function");
        };

        debug_assert_eq!(args.len(), self.args.len());

        for (arg_type, arg) in args.iter().zip(self.args.iter_mut()) {
            *arg.type_sig.borrow_mut() = Some(arg_type.clone());
        }
    }
}

impl<'a> Typed<'a> for FunctionArg<'a> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        self.type_sig
            .borrow()
            .clone()
            .ok_or(TypeEvalError::UndeterminableType(self.name.clone()))
    }

    fn specified_type(&self) -> Option<TypeSignature<'a>> {
        self.type_sig.borrow().clone()
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) {
        *self.type_sig.borrow_mut() = Some(new_type);
    }
}

impl<'a> Typed<'a> for FunctionCall<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match self.func.eval_type(symbols)? {
            TypeSignature::Function {
                args: _,
                return_type,
            } => Ok(*return_type),
            wrong_type => Err(TypeEvalError::CallNonFunction(wrong_type)),
        }
    }
}
