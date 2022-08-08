use std::fmt::Debug;

use id_arena::Id;

use crate::{
    ir::context::IrCtx, symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
    type_checker::function_body_type_eval::FunctionTypeError,
};

use super::{expression::Expr, identifier::Ident};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeSignature<'a> {
    id: Id<TypeSignatureValue<'a>>,
}

impl<'a> Into<Id<TypeSignatureValue<'a>>> for TypeSignature<'a> {
    fn into(self) -> Id<TypeSignatureValue<'a>> {
        self.id
    }
}

impl<'a> From<Id<TypeSignatureValue<'a>>> for TypeSignature<'a> {
    fn from(id: Id<TypeSignatureValue<'a>>) -> Self {
        Self { id }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TypeSignatureValue<'a> {
    Builtin(BuiltinType),
    Unresolved(Ident<'a>),
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

impl<'a> crate::ast::node::type_signature::TypeSignature<'a> {
    pub fn into_ir_type(self, ctx: &mut IrCtx<'a>) -> TypeSignature<'a> {
        let val = match self.value {
            crate::ast::node::type_signature::TypeSignatureValue::Base(base) => {
                TypeSignatureValue::Unresolved(ctx.make_unresolved_ident(base))
            }
            crate::ast::node::type_signature::TypeSignatureValue::Function {
                args,
                return_type,
            } => {
                let args = args.into_iter().map(|arg| arg.into_ir_type(ctx)).collect();
                let return_type = return_type.into_ir_type(ctx);

                TypeSignatureValue::Function { args, return_type }
            }
            crate::ast::node::type_signature::TypeSignatureValue::Tuple(types) => {
                let type_sigs = types.into_iter().map(|t| t.into_ir_type(ctx)).collect();
                TypeSignatureValue::Tuple(type_sigs)
            }
        };

        ctx.get_type_sig(val)
    }
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
        &self,
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
