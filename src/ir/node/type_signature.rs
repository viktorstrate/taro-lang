use std::fmt::Debug;

use id_arena::Id;

use crate::{ir::context::IrCtx, symbols::symbol_table::symbol_table_zipper::SymbolTableZipper};

use super::{expression::Expr, function::Function, identifier::Ident, NodeRef};

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

#[derive(Debug, Eq, Hash, Clone)]
pub enum TypeSignatureValue<'a> {
    Builtin(BuiltinType),
    Unresolved(Ident<'a>),
    TypeVariable(Id<TypeSignatureValue<'a>>),
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
    CallNonFunction(TypeSignature<'a>),
    FuncWrongNumberOfArgs {
        func: NodeRef<'a, Function<'a>>,
        expected: usize,
        actual: usize,
    },
    AccessNonStruct(TypeSignature<'a>),
    AccessNonTuple(TypeSignature<'a>),
    AccessNonEnum(TypeSignature<'a>),
    TupleAccessOutOfBounds {
        tuple_len: usize,
        access_item: usize,
    },
    UnknownIdent(Ident<'a>),
    UndeterminableType(Ident<'a>),
}

impl<'a> PartialEq for TypeSignatureValue<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Builtin(l0), Self::Builtin(r0)) => l0 == r0,
            (Self::Unresolved(l0), Self::Unresolved(r0)) => l0 == r0,
            (Self::TypeVariable(l0), Self::TypeVariable(r0)) => l0 == r0,
            (
                Self::Function {
                    args: l_args,
                    return_type: l_return_type,
                },
                Self::Function {
                    args: r_args,
                    return_type: r_return_type,
                },
            ) => {
                l_args.into_iter().zip(r_args).all(|(l, r)| l == r)
                    && l_return_type == r_return_type
            }
            (Self::Struct { name: l_name }, Self::Struct { name: r_name }) => l_name == r_name,
            (Self::Enum { name: l_name }, Self::Enum { name: r_name }) => l_name == r_name,
            (Self::Tuple(l0), Self::Tuple(r0)) => l0.into_iter().zip(r0).all(|(l, r)| l == r),
            _ => false,
        }
    }
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

impl<'a> TypeSignature<'a> {
    pub fn format(&self, ctx: &IrCtx<'a>) -> String {
        match ctx[*self].clone() {
            TypeSignatureValue::Builtin(builtin) => builtin.name().to_owned(),
            TypeSignatureValue::Unresolved(id) => format!("UNRESOLVED [{:?}]", ctx[id]),
            TypeSignatureValue::TypeVariable(var) => format!("TYPE_VAR [{:?}]", var.index()),
            TypeSignatureValue::Function { args, return_type } => format!(
                "({}) -> {}",
                args.into_iter()
                    .map(|arg| arg.format(ctx))
                    .intersperse(", ".to_owned())
                    .collect::<String>(),
                return_type.format(ctx)
            ),
            TypeSignatureValue::Struct { name } => format!("STRUCT {:?}", ctx[name]),
            TypeSignatureValue::Enum { name } => format!("ENUM {:?}", ctx[name]),
            TypeSignatureValue::Tuple(vals) => format!(
                "({})",
                vals.into_iter()
                    .map(|val| val.format(ctx))
                    .intersperse(", ".to_owned())
                    .collect::<String>()
            ),
        }
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
}

pub const BUILTIN_TYPES: &'static [BuiltinType] = &[
    BuiltinType::String,
    BuiltinType::Number,
    BuiltinType::Boolean,
    BuiltinType::Void,
];

impl BuiltinType {
    pub const fn name(&self) -> &'static str {
        match self {
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Boolean => "Boolean",
            BuiltinType::Void => "Void",
        }
    }
}
