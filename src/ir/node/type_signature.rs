use std::fmt::Debug;

use id_arena::Id;

use crate::{ir::context::IrCtx, symbols::symbol_table::symbol_table_zipper::SymbolTableZipper};

use super::{expression::Expr, identifier::Ident};

pub type TypeSignature<'a> = Id<TypeSignatureValue<'a>>;

#[derive(Debug)]
pub enum TypeSignatureValue<'a> {
    Builtin(BuiltinType),
    Unresolved(crate::ast::node::identifier::Ident<'a>),
    Function {
        args: Vec<TypeSignature<'a>>,
        return_type: TypeSignature<'a>,
    },
    Struct {
        name: Ident<'a>,
    },
    Enum {
        name: Ident<'a>,
    },
    Tuple(Vec<TypeSignature<'a>>),
}

#[derive(Debug)]
pub enum TypeEvalError<'a> {
    Expression(Expr<'a>),
    // FunctionType(FunctionTypeError<'a>),
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
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>>;

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        None
    }

    fn specify_type(
        &mut self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        Ok(())
    }
}

pub type Mutability = crate::ast::node::type_signature::Mutability;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum BuiltinType {
    String,
    Number,
    Boolean,
    Void,
    Untyped,
}

pub const BUILTIN_TYPES: &'static [BuiltinType] = &[
    BuiltinType::String,
    BuiltinType::Number,
    BuiltinType::Boolean,
    BuiltinType::Void,
    BuiltinType::Untyped,
];

impl BuiltinType {
    pub const fn name(&self) -> &'static str {
        match self {
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Boolean => "Boolean",
            BuiltinType::Void => "Void",
            BuiltinType::Untyped => "Untyped",
        }
    }
}
