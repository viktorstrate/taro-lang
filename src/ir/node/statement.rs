use super::{
    enumeration::Enum,
    expression::Expr,
    function::Function,
    identifier::{Ident, Identifiable},
    structure::Struct,
    type_signature::{Mutability, TypeSignature},
};

#[derive(Debug)]
pub enum Stmt<'a, 'ctx> {
    VariableDecl(VarDecl<'a, 'ctx>),
    FunctionDecl(Function<'a, 'ctx>),
    StructDecl(Struct<'a, 'ctx>),
    EnumDecl(Enum<'a, 'ctx>),
    Compound(Vec<Stmt<'a, 'ctx>>),
    Expression(Expr<'a, 'ctx>),
    Return(Expr<'a, 'ctx>),
}

#[derive(Debug)]
pub struct VarDecl<'a, 'ctx> {
    pub name: &'ctx Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<&'ctx TypeSignature<'a, 'ctx>>,
    pub value: Expr<'a, 'ctx>,
}

impl<'a, 'ctx> Identifiable<'a, 'ctx> for VarDecl<'a, 'ctx> {
    fn name(&self) -> &'ctx Ident<'a> {
        &self.name
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
