use id_arena::Id;

use crate::ir::context::IrCtx;

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::TypeSignature,
};

#[derive(Debug)]
pub struct Function<'a> {
    pub name: Ident<'a>,
    pub args: Vec<Id<FunctionArg<'a>>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub body: Id<Stmt<'a>>,
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

#[derive(Debug)]
pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub type_sig: Option<TypeSignature<'a>>,
}

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub func: Id<Expr<'a>>,
    pub params: Vec<Id<Expr<'a>>>,
}

impl<'a> Identifiable<'a> for Function<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

impl<'a> Identifiable<'a> for FunctionArg<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
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
