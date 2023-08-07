use std::{fmt::Debug, hash::Hash, ops::Deref, rc::Rc};

use id_arena::Id;

use crate::{
    error_message::error_formatter::Spanned,
    ir::{context::IrCtx, late_init::LateInit},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    enumeration::{Enum, EnumInit, EnumValue},
    escape_block::EscapeBlock,
    expression::Expr,
    external::ExternalObject,
    function::{Function, FunctionArg, FunctionCall},
    generics::GenericsDecl,
    identifier::{Ident, IdentParent},
    member_access::UnresolvedMemberAccess,
    statement::VarDecl,
    structure::{Struct, StructAttr, StructInit},
    traits::{Trait, TraitFuncAttr},
    tuple::{Tuple, TupleAccess},
    NodeRef,
};

#[derive(Debug, Clone)]
pub struct TypeSignature<'a> {
    pub id: Id<TypeSignatureValue<'a>>,
    pub context: Rc<TypeSignatureContext<'a>>,
}

impl<'a> Eq for TypeSignature<'a> {}

impl<'a> PartialEq for TypeSignature<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a> Hash for TypeSignature<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct TypeSignatureContext<'a> {
    pub parent: TypeSignatureParent<'a>,
    pub type_span: Option<Span<'a>>,
}

impl<'a> TypeSignatureContext<'a> {
    #[inline]
    pub fn alloc(self) -> Rc<Self> {
        Rc::new(self)
    }
}

#[derive(Debug, Clone)]
pub enum TypeSignatureParent<'a> {
    Builtin,
    Generic(NodeRef<'a, GenericsDecl<'a>>),
    VarDeclSig(NodeRef<'a, VarDecl<'a>>),
    Enum(NodeRef<'a, Enum<'a>>),
    EnumValue(NodeRef<'a, EnumValue<'a>>),
    EnumInit(NodeRef<'a, EnumInit<'a>>),
    Expr(NodeRef<'a, Expr<'a>>),
    Function(NodeRef<'a, Function<'a>>),
    FunctionArg {
        parent_func: TypeSignature<'a>,
    },
    FunctionReturn {
        parent_func: TypeSignature<'a>,
    },
    FunctionDefArg(NodeRef<'a, FunctionArg<'a>>),
    FunctionDefReturn(NodeRef<'a, Function<'a>>),
    Struct(NodeRef<'a, Struct<'a>>),
    StructInit(NodeRef<'a, StructInit<'a>>),
    StructAttr(NodeRef<'a, StructAttr<'a>>),
    Trait(NodeRef<'a, Trait<'a>>),
    Tuple(NodeRef<'a, Tuple<'a>>),
    TupleItem {
        attr: usize,
        parent_tuple: TypeSignature<'a>,
    },
    EscapeBlock(NodeRef<'a, EscapeBlock<'a>>),
    MemberAccess(NodeRef<'a, UnresolvedMemberAccess<'a>>),
    ExternObjType(NodeRef<'a, ExternalObject<'a>>),
    TraitFuncAttr(NodeRef<'a, TraitFuncAttr<'a>>),
}

impl<'a> Into<Id<TypeSignatureValue<'a>>> for TypeSignature<'a> {
    fn into(self) -> Id<TypeSignatureValue<'a>> {
        self.id
    }
}

impl<'a> Into<Id<TypeSignatureValue<'a>>> for &TypeSignature<'a> {
    fn into(self) -> Id<TypeSignatureValue<'a>> {
        self.id
    }
}

#[derive(Debug, Eq, Hash, Clone)]
pub enum TypeSignatureValue<'a> {
    Builtin(BuiltinType),
    Unresolved(Ident<'a>),
    TypeVariable(Id<TypeSignatureValue<'a>>),
    Function {
        args: LateInit<Vec<TypeSignature<'a>>>,
        return_type: LateInit<TypeSignature<'a>>,
    },
    Struct {
        name: Ident<'a>,
    },
    Enum {
        name: Ident<'a>,
    },
    Tuple(LateInit<Vec<TypeSignature<'a>>>),
    Trait {
        name: Ident<'a>,
    },
}

impl<'a> Spanned<'a> for TypeSignature<'a> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        if let Some(span) = self.context.type_span.clone() {
            return Some(span);
        }

        let node_span = match &self.context.parent {
            TypeSignatureParent::Builtin => None,
            TypeSignatureParent::Generic(gen_decl) => gen_decl.get_span(ctx),
            TypeSignatureParent::VarDeclSig(var) => ctx[*var].name.get_span(ctx),
            TypeSignatureParent::Enum(_) => todo!(),
            TypeSignatureParent::EnumValue(_) => todo!(),
            TypeSignatureParent::EnumInit(_) => todo!(),
            TypeSignatureParent::Expr(expr) => expr.get_span(ctx),
            TypeSignatureParent::Function(func) => Some(ctx[*func].span.clone()),
            TypeSignatureParent::FunctionArg { parent_func: _ } => todo!(),
            TypeSignatureParent::FunctionReturn { parent_func: _ } => todo!(),
            TypeSignatureParent::FunctionDefArg(arg) => arg.get_span(ctx),
            TypeSignatureParent::FunctionDefReturn(_) => todo!(),
            TypeSignatureParent::Struct(st) => st.get_span(ctx),
            TypeSignatureParent::StructInit(st_init) => st_init.get_span(ctx),
            TypeSignatureParent::StructAttr(st_attr) => st_attr.get_span(ctx),
            TypeSignatureParent::Tuple(tup) => tup.get_span(ctx),
            TypeSignatureParent::TupleItem {
                attr: _,
                parent_tuple: _,
            } => todo!(),
            TypeSignatureParent::EscapeBlock(esc) => esc.get_span(ctx),
            TypeSignatureParent::MemberAccess(mem_acc) => mem_acc.get_span(ctx),
            TypeSignatureParent::ExternObjType(obj) => obj.get_span(ctx),
            TypeSignatureParent::TraitFuncAttr(_) => todo!(),
            TypeSignatureParent::Trait(tr) => tr.get_span(ctx),
        };

        if node_span.is_some() {
            return node_span;
        }

        match &ctx[self] {
            TypeSignatureValue::Builtin(_) => None,
            TypeSignatureValue::Unresolved(id) => id.get_span(ctx),
            TypeSignatureValue::TypeVariable(_) => todo!(),
            TypeSignatureValue::Function {
                args: _,
                return_type: _,
            } => todo!(),
            TypeSignatureValue::Struct { name } => name.get_span(ctx),
            TypeSignatureValue::Enum { name } => name.get_span(ctx),
            TypeSignatureValue::Tuple(_) => todo!(),
            TypeSignatureValue::Trait { name } => name.get_span(ctx),
        }
    }
}

#[derive(Debug)]
pub enum TypeEvalError<'a> {
    CallNonFunction(NodeRef<'a, FunctionCall<'a>>, TypeSignature<'a>),
    FuncWrongNumberOfArgs {
        func: NodeRef<'a, Function<'a>>,
        expected: usize,
        actual: usize,
    },
    AccessNonStruct(TypeSignature<'a>),
    AccessNonTuple(TypeSignature<'a>),
    AccessNonEnum(TypeSignature<'a>),
    TupleAccessOutOfBounds(NodeRef<'a, TupleAccess<'a>>, TypeSignature<'a>),
    UnknownIdent(Ident<'a>),
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
                (**l_args)
                    .clone()
                    .into_iter()
                    .zip(r_args.deref())
                    .all(|(l, r)| l == *r)
                    && l_return_type == r_return_type
            }
            (Self::Struct { name: l_name }, Self::Struct { name: r_name }) => l_name == r_name,
            (Self::Enum { name: l_name }, Self::Enum { name: r_name }) => l_name == r_name,
            (Self::Tuple(l0), Self::Tuple(r0)) => (**l0)
                .clone()
                .into_iter()
                .zip(r0.deref())
                .all(|(l, r)| l == *r),
            _ => false,
        }
    }
}

impl<'a> crate::ast::node::type_signature::TypeSignature<'a> {
    pub fn into_ir_type(
        self,
        ctx: &mut IrCtx<'a>,
        parent: TypeSignatureParent<'a>,
    ) -> TypeSignature<'a> {
        let type_ctx = TypeSignatureContext {
            parent: parent,
            type_span: Some(self.span),
        }
        .alloc();

        match self.value {
            crate::ast::node::type_signature::TypeSignatureValue::Base(base) => {
                let ident = ctx.make_unresolved_ident(base, LateInit::empty());
                let type_sig = ctx.get_type_sig(TypeSignatureValue::Unresolved(ident), type_ctx);

                match &mut ctx[&type_sig] {
                    TypeSignatureValue::Unresolved(id) => {
                        id.parent = IdentParent::TypeSigName(type_sig.id).into()
                    }
                    _ => unreachable!(),
                }

                type_sig
            }
            crate::ast::node::type_signature::TypeSignatureValue::Function {
                args,
                return_type,
            } => {
                let func = ctx.get_type_sig(
                    TypeSignatureValue::Function {
                        args: LateInit::empty(),
                        return_type: LateInit::empty(),
                    },
                    type_ctx,
                );

                let new_args = args
                    .into_iter()
                    .enumerate()
                    .map(|(_i, arg)| {
                        arg.into_ir_type(
                            ctx,
                            TypeSignatureParent::FunctionArg {
                                parent_func: func.clone(),
                            },
                        )
                    })
                    .collect::<Vec<TypeSignature<'a>>>()
                    .into();

                let new_return = return_type
                    .into_ir_type(
                        ctx,
                        TypeSignatureParent::FunctionReturn {
                            parent_func: func.clone(),
                        },
                    )
                    .into();

                match &mut ctx[&func] {
                    TypeSignatureValue::Function {
                        args: ags,
                        return_type: ret_type,
                    } => {
                        *ags = new_args;
                        *ret_type = new_return;
                    }
                    _ => unreachable!(),
                }

                func
            }
            crate::ast::node::type_signature::TypeSignatureValue::Tuple(types) => {
                // let tup = match &type_ctx.parent {
                //     TypeSignatureParent::Tuple(tup) => *tup,
                //     _ => unreachable!(),
                // };

                let tup = ctx.get_type_sig(TypeSignatureValue::Tuple(LateInit::empty()), type_ctx);

                let type_sigs = types
                    .into_iter()
                    .enumerate()
                    .map(|(i, t)| {
                        t.into_ir_type(
                            ctx,
                            TypeSignatureParent::TupleItem {
                                attr: i,
                                parent_tuple: tup.clone(),
                            },
                        )
                    })
                    .collect::<Vec<TypeSignature<'a>>>()
                    .into();

                match &mut ctx[&tup] {
                    TypeSignatureValue::Tuple(t) => {
                        *t = type_sigs;
                    }
                    _ => unreachable!(),
                }

                tup
            }
        }
    }
}

impl<'a> TypeSignature<'a> {
    pub fn format(&self, ctx: &IrCtx<'a>) -> String {
        match ctx[self].clone() {
            TypeSignatureValue::Builtin(builtin) => builtin.name().to_owned(),
            TypeSignatureValue::Unresolved(id) => id.value(ctx).unwrap().to_owned(),
            TypeSignatureValue::TypeVariable(_var) => "[unknown]".to_owned(),
            TypeSignatureValue::Function { args, return_type } => format!(
                "({}) -> {}",
                (*args)
                    .clone()
                    .into_iter()
                    .map(|arg| arg.format(ctx))
                    .intersperse(", ".to_owned())
                    .collect::<String>(),
                return_type.format(ctx)
            ),
            TypeSignatureValue::Struct { name } => format!("[struct {}]", name.value(ctx).unwrap()),
            TypeSignatureValue::Enum { name } => format!("[enum {}]", name.value(ctx).unwrap()),
            TypeSignatureValue::Tuple(vals) => format!(
                "({})",
                (*vals)
                    .clone()
                    .into_iter()
                    .map(|val| val.format(ctx))
                    .intersperse(", ".to_owned())
                    .collect::<String>()
            ),
            TypeSignatureValue::Trait { name } => format!("[trait {}]", name.value(ctx).unwrap()),
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

    fn specified_type(&self, ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
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
