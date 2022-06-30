use crate::symbols::{
    builtin_types::BuiltinType, symbol_table::symbol_table_zipper::SymbolTableZipper,
    symbol_table::SymbolValue,
};

use super::{
    assignment::Assignment,
    escape_block::EscapeBlock,
    function::{Function, FunctionCall},
    identifier::Ident,
    structure::{StructAccess, StructInit},
    type_signature::{TypeEvalError, TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Function(Function<'a>),
    FunctionCall(Box<FunctionCall<'a>>),
    Identifier(Ident<'a>),
    StructInit(StructInit<'a>),
    StructAccess(StructAccess<'a>),
    EscapeBlock(EscapeBlock<'a>),
    Assignment(Box<Assignment<'a>>),
}

impl<'a> Typed<'a> for Expr<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match self {
            Expr::StringLiteral(_) => Ok(BuiltinType::String.type_sig()),
            Expr::NumberLiteral(_) => Ok(BuiltinType::Number.type_sig()),
            Expr::BoolLiteral(_) => Ok(BuiltinType::Boolean.type_sig()),
            Expr::Function(func) => func.eval_type(symbols),
            Expr::FunctionCall(call) => call.eval_type(symbols),
            Expr::Identifier(ident) => {
                let sym_val = symbols
                    .lookup(ident)
                    .ok_or(TypeEvalError::UnknownIdentifier(ident.clone()))?;

                match sym_val {
                    SymbolValue::BuiltinType(builtin) => Ok(TypeSignature::Base(builtin.clone())),
                    SymbolValue::VarDecl(var_decl) => var_decl.clone().value.eval_type(symbols),
                    SymbolValue::FuncDecl(func_decl) => Ok(func_decl
                        .clone()
                        .eval_type(symbols)
                        .expect("function type sig always succeeds")),
                    SymbolValue::FuncArg(arg) => Ok(arg.type_sig.clone()),
                    SymbolValue::StructDecl(st) => st.clone().eval_type(symbols),
                    SymbolValue::StructAttr(attr) => attr.clone().eval_type(symbols),
                }
            }
            Expr::StructInit(struct_init) => struct_init.eval_type(symbols),
            Expr::StructAccess(struct_access) => struct_access.eval_type(symbols),
            Expr::EscapeBlock(block) => block.eval_type(symbols),
            Expr::Assignment(asg) => asg.rhs.eval_type(symbols),
        }
    }

    fn specified_type(&self) -> Option<&TypeSignature<'a>> {
        match self {
            Expr::StringLiteral(_) => None,
            Expr::NumberLiteral(_) => None,
            Expr::BoolLiteral(_) => None,
            Expr::Function(func) => func.specified_type(),
            Expr::FunctionCall(call) => call.specified_type(),
            Expr::Identifier(_) => None,
            Expr::StructInit(st_init) => st_init.specified_type(),
            Expr::StructAccess(_) => None,
            Expr::EscapeBlock(block) => block.specified_type(),
            Expr::Assignment(_) => None,
        }
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) {
        match self {
            Expr::StringLiteral(_) => {}
            Expr::NumberLiteral(_) => {}
            Expr::BoolLiteral(_) => {}
            Expr::Function(func) => func.specify_type(new_type),
            Expr::FunctionCall(call) => call.specify_type(new_type),
            Expr::Identifier(_) => {}
            Expr::StructInit(st_init) => st_init.specify_type(new_type),
            Expr::StructAccess(_) => {}
            Expr::EscapeBlock(block) => block.specify_type(new_type),
            Expr::Assignment(_) => {}
        }
    }
}
