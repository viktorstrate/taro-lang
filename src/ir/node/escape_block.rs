use crate::symbols::{
    builtin_types::BuiltinType, symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::type_signature::{TypeEvalError, TypeSignature, Typed};

#[derive(Debug, Clone)]
pub struct EscapeBlock<'a> {
    pub content: &'a str,
    pub type_sig: Option<TypeSignature<'a>>,
}

impl<'a> Typed<'a> for EscapeBlock<'a> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, super::type_signature::TypeEvalError<'a>> {
        if let Some(sig) = &self.type_sig {
            Ok(sig.clone())
        } else {
            Ok(BuiltinType::Untyped.type_sig())
        }
    }

    fn specified_type(&self) -> Option<TypeSignature<'a>> {
        self.type_sig.clone()
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
        self.type_sig = Some(new_type);
        Ok(())
    }
}
