use id_arena::Id;

use super::{expression::Expr, type_signature::TypeSignature};

#[derive(Debug)]
pub struct Tuple<'a> {
    pub values: Vec<Id<Expr<'a>>>,
    pub type_sig: Option<TypeSignature<'a>>,
}

#[derive(Debug)]
pub struct TupleAccess<'a> {
    pub tuple_expr: Id<Expr<'a>>,
    pub attr: usize,
}

// impl<'a> Typed<'a> for Tuple<'a> {
//     fn eval_type(
//         &self,
//         symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         let types = self
//             .values
//             .iter()
//             .map(|val| val.eval_type(symbols))
//             .collect::<Result<Vec<_>, _>>()?;

//         Ok(TypeSignature::Tuple(types))
//     }

//     fn specified_type(&self) -> Option<TypeSignature<'a>> {
//         self.type_sig.clone()
//     }

//     fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
//         match &new_type {
//             TypeSignature::Tuple(vals) => assert_eq!(vals.len(), self.values.len()),
//             _ => assert!(false),
//         }

//         self.type_sig = Some(new_type);
//         Ok(())
//     }
// }

// impl<'a> Typed<'a> for TupleAccess<'a> {
//     fn eval_type(
//         &self,
//         symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         match self.tuple_expr.eval_type(symbols)? {
//             TypeSignature::Tuple(tuple) => {
//                 tuple
//                     .get(self.attr)
//                     .cloned()
//                     .ok_or(TypeEvalError::TupleAccessOutOfBounds {
//                         tuple_len: tuple.len(),
//                         access_item: self.attr,
//                     })
//             }
//             val => Err(TypeEvalError::AccessNonTuple(val)),
//         }
//     }
// }
