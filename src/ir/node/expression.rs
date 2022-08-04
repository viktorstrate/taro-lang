use super::{
    assignment::Assignment,
    escape_block::EscapeBlock,
    function::{Function, FunctionCall},
    identifier::Ident,
    structure::{StructAccess, StructInit},
    tuple::{Tuple, TupleAccess},
};

#[derive(Debug)]
pub enum Expr<'a, 'ctx> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Function(&'ctx mut Function<'a, 'ctx>),
    FunctionCall(&'ctx mut FunctionCall<'a, 'ctx>),
    Identifier(Ident<'a, 'ctx>),
    StructInit(&'ctx mut StructInit<'a, 'ctx>),
    StructAccess(&'ctx mut StructAccess<'a, 'ctx>),
    TupleAccess(&'ctx mut TupleAccess<'a, 'ctx>),
    EscapeBlock(&'ctx mut EscapeBlock<'a, 'ctx>),
    Assignment(&'ctx mut Assignment<'a, 'ctx>),
    Tuple(&'ctx mut Tuple<'a, 'ctx>),
}

// impl<'a> Typed<'a> for Expr<'a> {
//     fn eval_type(
//         &self,
//         symbols: &mut SymbolTableZipper<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         match self {
//             Expr::StringLiteral(_) => Ok(BuiltinType::String.type_sig()),
//             Expr::NumberLiteral(_) => Ok(BuiltinType::Number.type_sig()),
//             Expr::BoolLiteral(_) => Ok(BuiltinType::Boolean.type_sig()),
//             Expr::Function(func) => func.eval_type(symbols),
//             Expr::FunctionCall(call) => call.eval_type(symbols),
//             Expr::Identifier(ident) => {
//                 let sym_val = symbols
//                     .lookup(ident)
//                     .ok_or(TypeEvalError::UnknownIdentifier(ident.clone()))?;

//                 sym_val.clone().eval_type(symbols)
//             }
//             Expr::StructInit(struct_init) => struct_init.eval_type(symbols),
//             Expr::StructAccess(struct_access) => struct_access.eval_type(symbols),
//             Expr::EscapeBlock(block) => block.eval_type(symbols),
//             Expr::Assignment(asg) => asg.rhs.eval_type(symbols),
//             Expr::Tuple(tup) => tup.eval_type(symbols),
//             Expr::TupleAccess(tup_acc) => tup_acc.eval_type(symbols),
//         }
//     }

//     fn specified_type(&self) -> Option<TypeSignature<'a>> {
//         match self {
//             Expr::StringLiteral(_) => None,
//             Expr::NumberLiteral(_) => None,
//             Expr::BoolLiteral(_) => None,
//             Expr::Function(func) => func.specified_type(),
//             Expr::FunctionCall(call) => call.specified_type(),
//             Expr::Identifier(_) => None,
//             Expr::StructInit(st_init) => st_init.specified_type(),
//             Expr::StructAccess(_) => None,
//             Expr::EscapeBlock(block) => block.specified_type(),
//             Expr::Assignment(_) => None,
//             Expr::Tuple(tup) => tup.specified_type(),
//             Expr::TupleAccess(tup_acc) => tup_acc.specified_type(),
//         }
//     }

//     fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
//         match self {
//             Expr::StringLiteral(_) => Ok(()),
//             Expr::NumberLiteral(_) => Ok(()),
//             Expr::BoolLiteral(_) => Ok(()),
//             Expr::Function(func) => func.specify_type(new_type),
//             Expr::FunctionCall(call) => call.specify_type(new_type),
//             Expr::Identifier(_) => Ok(()),
//             Expr::StructInit(st_init) => st_init.specify_type(new_type),
//             Expr::StructAccess(_) => Ok(()),
//             Expr::EscapeBlock(block) => block.specify_type(new_type),
//             Expr::Assignment(_) => Ok(()),
//             Expr::Tuple(tup) => tup.specify_type(new_type),
//             Expr::TupleAccess(tup_acc) => tup_acc.specify_type(new_type),
//         }
//     }
// }
