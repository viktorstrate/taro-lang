use crate::symbols::symbol_table::symbol_table_zipper::SymbolTableZipper;

use super::{
    expression::Expr,
    type_signature::{TypeEvalError, TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub struct Tuple<'a> {
    pub values: Vec<Expr<'a>>,
    pub type_sig: Option<TypeSignature<'a>>,
}

impl<'a> Typed<'a> for Tuple<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let types = self
            .values
            .iter()
            .map(|val| val.eval_type(symbols))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TypeSignature::Tuple(types))
    }

    fn specified_type(&self) -> Option<TypeSignature<'a>> {
        self.type_sig.clone()
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
        match &new_type {
            TypeSignature::Tuple(vals) => assert_eq!(vals.len(), self.values.len()),
            _ => assert!(false),
        }

        self.type_sig = Some(new_type);
        Ok(())
    }
}
