use id_arena::Id;

use crate::ir::context::IrCtx;

use super::{
    enumeration::Enum,
    expression::Expr,
    function::Function,
    identifier::{Ident, Identifiable},
    structure::Struct,
    type_signature::{Mutability, TypeSignature},
};

#[derive(Debug)]
pub enum Stmt<'a> {
    VariableDecl(Id<VarDecl<'a>>),
    FunctionDecl(Id<Function<'a>>),
    StructDecl(Id<Struct<'a>>),
    EnumDecl(Id<Enum<'a>>),
    Compound(Vec<Id<Stmt<'a>>>),
    Expression(Id<Expr<'a>>),
    Return(Id<Expr<'a>>),
}

#[derive(Debug)]
pub struct VarDecl<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub value: Id<Expr<'a>>,
}

impl<'a> Identifiable<'a> for VarDecl<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

// impl<'a> Typed<'a> for VarDecl<'a> {
//     fn eval_type(
//         &self,
//         symbols: &mut crate::symbols::symbol_table::symbol_table_zipper::SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         self.value.eval_type(symbols)
//     }

//     fn specified_type(&self) -> Option<TypeSignature<'a>> {
//         self.type_sig.clone()
//     }

//     fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
//         self.type_sig = Some(new_type);
//         Ok(())
//     }
// }
