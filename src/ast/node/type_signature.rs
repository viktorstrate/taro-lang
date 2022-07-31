use std::fmt::Debug;

use crate::{
    ir::ref_generator::RefID, parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
    type_checker::function_body_type_eval::FunctionTypeError,
};

use super::{expression::Expr, identifier::Ident};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeSignature<'a> {
    pub span: Span<'a>,
    pub value: TypeSignatureValue<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSignatureValue<'a> {
    Base(Ident<'a>),
    Function {
        args: Vec<TypeSignature<'a>>,
        return_type: Box<TypeSignature<'a>>,
    },
    Struct {
        name: Ident<'a>,
        ref_id: RefID,
    },
    Enum {
        name: Ident<'a>,
        ref_id: RefID,
    },
    Tuple(Vec<TypeSignature<'a>>),
}

#[derive(Debug)]
pub enum TypeEvalError<'a> {
    Expression(Expr<'a>),
    FunctionType(FunctionTypeError<'a>),
    CallNonFunction(TypeSignature<'a>),
    AccessNonStruct(TypeSignature<'a>),
    AccessNonTuple(TypeSignature<'a>),
    TupleAccessOutOfBounds {
        tuple_len: usize,
        access_item: usize,
    },
    UnknownIdentifier(Ident<'a>),
    UndeterminableType(Ident<'a>),
}

#[allow(unused_variables)]
pub trait Typed<'a>: Debug {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>>;

    fn specified_type(&self) -> Option<TypeSignature<'a>> {
        None
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) -> Result<(), TypeEvalError<'a>> {
        Ok(())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Mutability {
    Mutable,
    Immutable,
}

impl From<bool> for Mutability {
    fn from(val: bool) -> Self {
        if val {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }
    }
}

impl Into<bool> for Mutability {
    fn into(self) -> bool {
        self == Mutability::Mutable
    }
}
