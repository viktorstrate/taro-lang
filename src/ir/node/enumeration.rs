use id_arena::Id;

use crate::ir::context::IrCtx;

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    type_signature::TypeSignature,
};

#[derive(Debug, PartialEq)]
pub struct Enum<'a> {
    pub name: Ident<'a>,
    pub values: Vec<Id<EnumValue<'a>>>,
}

#[derive(Debug, PartialEq)]
pub struct EnumValue<'a> {
    pub name: Ident<'a>,
    pub items: Vec<TypeSignature<'a>>,
}

pub struct EnumInit<'a> {
    pub enum_name: Option<Ident<'a>>,
    pub enum_value: Ident<'a>,
    pub items: Vec<Id<Expr<'a>>>,
}

impl<'a> Identifiable<'a> for Enum<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

impl<'a> Identifiable<'a> for EnumValue<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

// impl<'a> Typed<'a> for Enum<'a> {
//     fn eval_type(
//         &self,
//         _symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         Ok(TypeSignature::Enum {
//             name: self.name.clone(),
//             ref_id: self.ref_id,
//         })
//     }
// }

// impl<'a> Typed<'a> for EnumValue<'a> {
//     fn eval_type(
//         &self,
//         _symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         Ok(self.specified_type().unwrap())
//     }

//     fn specified_type(&self) -> Option<TypeSignature<'a>> {
//         Some(TypeSignature::Tuple(self.items.clone()))
//     }

//     fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
//         let TypeSignature::Tuple(tuple) = new_type else {
//             unreachable!("specified type expected to be tuple");
//         };

//         assert_eq!(tuple.len(), self.items.len());

//         self.items = tuple;
//         Ok(())
//     }
// }
