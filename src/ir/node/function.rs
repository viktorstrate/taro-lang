

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::{TypeSignature},
};

#[derive(Debug)]
pub struct Function<'a, 'ctx> {
    pub name: &'ctx Ident<'a>,
    pub args: Vec<&'ctx FunctionArg<'a, 'ctx>>,
    pub return_type: Option<&'ctx TypeSignature<'a, 'ctx>>,
    pub body: &'ctx mut Stmt<'a, 'ctx>,
}

// impl<'a> Function<'a> {
//     pub fn calculate_type_sig(
//         args: &Vec<FunctionArg<'a>>,
//         return_type: &Option<TypeSignature<'a>>,
//     ) -> Option<TypeSignature<'a>> {
//         let arg_types = args
//             .iter()
//             .map(|arg| arg.type_sig.borrow().clone())
//             .collect::<Option<Vec<_>>>();

//         match (arg_types, return_type) {
//             (Some(args), Some(return_type)) => Some(TypeSignature::Function {
//                 args,
//                 return_type: Box::new(return_type.clone()),
//             }),
//             _ => None,
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct FunctionArg<'a, 'ctx> {
    pub name: &'ctx Ident<'a>,
    pub type_sig: Option<&'ctx TypeSignature<'a, 'ctx>>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall<'a, 'ctx> {
    pub func: &'ctx Expr<'a, 'ctx>,
    pub params: Vec<&'ctx Expr<'a, 'ctx>>,
}

impl<'a, 'ctx> Identifiable<'a, 'ctx> for Function<'a, 'ctx> {
    fn name(&self) -> &'ctx Ident<'a> {
        &self.name
    }
}

impl<'a, 'ctx> Identifiable<'a, 'ctx> for FunctionArg<'a, 'ctx> {
    fn name(&self) -> &'ctx Ident<'a> {
        &self.name
    }
}

// impl<'a> Typed<'a> for Function<'a> {
//     fn eval_type(
//         &self,
//         symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         let args = self
//             .args
//             .iter()
//             .map(|arg| arg.eval_type(symbols))
//             .collect::<Result<Vec<_>, _>>()?;

//         symbols
//             .enter_scope(self.name.clone())
//             .expect("function should be located in current scope");

//         let return_type =
//             eval_func_body_type_sig(symbols, self).map_err(TypeEvalError::FunctionType)?;

//         symbols.exit_scope().unwrap();

//         Ok(TypeSignature::Function {
//             args,
//             return_type: Box::new(return_type),
//         })
//     }

//     fn specified_type(&self) -> Option<TypeSignature<'a>> {
//         Self::calculate_type_sig(&self.args, &self.return_type)
//     }

//     fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
//         let TypeSignature::Function { args, return_type: _ } = &new_type else {
//             unreachable!("specified type expected to be function");
//         };

//         if args.len() != self.args.len() {
//             return Err(TypeEvalError::FunctionType(
//                 FunctionTypeError::WrongNumberOfArgs {
//                     func: self.clone(),
//                     expected: args.len(),
//                     actual: self.args.len(),
//                 },
//             ));
//         }

//         for (arg_type, arg) in args.iter().zip(self.args.iter_mut()) {
//             *arg.type_sig.borrow_mut() = Some(arg_type.clone());
//         }

//         Ok(())
//     }
// }

// impl<'a> Typed<'a> for FunctionArg<'a> {
//     fn eval_type(
//         &self,
//         _symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         self.type_sig
//             .borrow()
//             .clone()
//             .ok_or(TypeEvalError::UndeterminableType(self.name.clone()))
//     }

//     fn specified_type(&self) -> Option<TypeSignature<'a>> {
//         self.type_sig.borrow().clone()
//     }

//     fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
//         *self.type_sig.borrow_mut() = Some(new_type);
//         Ok(())
//     }
// }

// impl<'a> Typed<'a> for FunctionCall<'a> {
//     fn eval_type(
//         &self,
//         symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         match self.func.eval_type(symbols)? {
//             TypeSignature::Function {
//                 args: _,
//                 return_type,
//             } => Ok(*return_type),
//             wrong_type => Err(TypeEvalError::CallNonFunction(wrong_type)),
//         }
//     }
// }
