use id_arena::Id;

use crate::{
    error_message::error_formatter::Spanned,
    ir::{context::IrCtx, late_init::LateInit},
    parser::Span,
};
use std::{fmt::Debug, hash::Hash};

use super::{
    enumeration::{Enum, EnumInit, EnumValue},
    expression::Expr,
    function::{Function, FunctionArg},
    member_access::UnresolvedMemberAccess,
    statement::VarDecl,
    structure::{Struct, StructAccess, StructAttr, StructInit, StructInitValue},
    type_signature::{BuiltinType, TypeSignatureValue},
    NodeRef,
};

pub trait Identifiable<'a> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a>;
}

#[derive(Debug, Clone, Copy)]
pub struct Ident<'a> {
    pub id: Id<IdentValue<'a>>,
    pub parent: LateInit<IdentParent<'a>>,
}

impl<'a> Eq for Ident<'a> {}

impl<'a> PartialEq for Ident<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a> Hash for Ident<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<'a> Into<Id<IdentValue<'a>>> for Ident<'a> {
    fn into(self) -> Id<IdentValue<'a>> {
        self.id
    }
}

impl<'a> Spanned<'a> for Ident<'a> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        match &ctx[*self] {
            IdentValue::Resolved(id) => match id {
                ResolvedIdentValue::Named { def_span, name: _ } => Some(def_span.clone()),
                ResolvedIdentValue::Anonymous => None,
                ResolvedIdentValue::BuiltinType(_) => None,
            },
            IdentValue::Unresolved(id) => Some(id.span.clone()),
        }
    }
}

// impl<'a> From<Id<IdentValue<'a>>> for Ident<'a> {
//     fn from(id: Id<IdentValue<'a>>) -> Self {
//         Self { id }
//     }
// }

#[derive(Debug, Clone)]
pub enum IdentValue<'a> {
    Resolved(ResolvedIdentValue<'a>),
    Unresolved(crate::ast::node::identifier::Ident<'a>),
}

#[derive(Debug, Clone)]
pub enum ResolvedIdentValue<'a> {
    Named { def_span: Span<'a>, name: &'a str },
    Anonymous,
    BuiltinType(BuiltinType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentKey<'a> {
    Named(&'a str),
    Unnamed(Ident<'a>),
}

impl<'a> IdentKey<'a> {
    pub fn from_ident(ctx: &IrCtx<'a>, ident: Ident<'a>) -> IdentKey<'a> {
        match &ctx[ident] {
            IdentValue::Resolved(val) => match val {
                ResolvedIdentValue::Named { def_span: _, name } => IdentKey::Named(name),
                ResolvedIdentValue::Anonymous => IdentKey::Unnamed(ident),
                ResolvedIdentValue::BuiltinType(builtin) => IdentKey::Named(builtin.name()),
            },
            IdentValue::Unresolved(ast_ident) => IdentKey::Named(ast_ident.value),
        }
    }

    pub fn idents_eq(ctx: &IrCtx<'a>, a: Ident<'a>, b: Ident<'a>) -> bool {
        let key_a = IdentKey::from_ident(ctx, a);
        let key_b = IdentKey::from_ident(ctx, b);

        key_a == key_b
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IdentParent<'a> {
    StructDeclName(NodeRef<'a, Struct<'a>>),
    StructDeclAttrName(NodeRef<'a, StructAttr<'a>>),
    StructInitValueName(NodeRef<'a, StructInitValue<'a>>),
    StructInitStructName(NodeRef<'a, StructInit<'a>>),
    StructInitScopeName(NodeRef<'a, StructInit<'a>>),
    StructAccessAttrName(NodeRef<'a, StructAccess<'a>>),
    EnumDeclName(NodeRef<'a, Enum<'a>>),
    EnumDeclValueName(NodeRef<'a, EnumValue<'a>>),
    EnumInitValueName(NodeRef<'a, EnumInit<'a>>),
    EnumInitEnumName(NodeRef<'a, EnumInit<'a>>),
    VarDeclName(NodeRef<'a, VarDecl<'a>>),
    FuncDeclName(NodeRef<'a, Function<'a>>),
    FuncDeclArgName(NodeRef<'a, FunctionArg<'a>>),
    IdentExpr(NodeRef<'a, Expr<'a>>),
    TypeSigName(Id<TypeSignatureValue<'a>>),
    MemberAccessMemberName(NodeRef<'a, UnresolvedMemberAccess<'a>>),
    BuiltinIdent,
}

impl<'a> IdentParent<'a> {
    pub fn change_ident(&self, ctx: &mut IrCtx<'a>, new_ident: Ident<'a>) {
        match self {
            IdentParent::StructDeclName(st_decl) => ctx[*st_decl].name = new_ident.into(),
            IdentParent::StructDeclAttrName(st_attr) => ctx[*st_attr].name = new_ident.into(),
            IdentParent::StructInitValueName(st_val) => ctx[*st_val].name = new_ident.into(),
            IdentParent::StructInitStructName(st_init) => {
                ctx[*st_init].struct_name = new_ident.into()
            }
            IdentParent::StructInitScopeName(st_init) => {
                ctx[*st_init].scope_name = new_ident.into()
            }
            IdentParent::StructAccessAttrName(st_acc) => ctx[*st_acc].attr_name = new_ident,
            IdentParent::EnumDeclName(enm) => ctx[*enm].name = new_ident.into(),
            IdentParent::EnumDeclValueName(enm_val) => ctx[*enm_val].name = new_ident.into(),
            IdentParent::EnumInitValueName(enm_init) => ctx[*enm_init].enum_value = new_ident,
            IdentParent::EnumInitEnumName(enm_init) => ctx[*enm_init].enum_name = new_ident,
            IdentParent::VarDeclName(var_decl) => ctx[*var_decl].name = new_ident.into(),
            IdentParent::FuncDeclName(func) => ctx[*func].name = new_ident.into(),
            IdentParent::FuncDeclArgName(func_arg) => ctx[*func_arg].name = new_ident.into(),
            IdentParent::IdentExpr(id_expr) => match &mut ctx[*id_expr] {
                Expr::Identifier(id, _) => *id = new_ident.into(),
                _ => unreachable!(),
            },
            IdentParent::TypeSigName(type_sig) => match &mut ctx.types[*type_sig] {
                TypeSignatureValue::Unresolved(ident) => *ident = new_ident.into(),
                TypeSignatureValue::Enum { name } => *name = new_ident,
                TypeSignatureValue::Struct { name } => *name = new_ident,
                _ => unreachable!(),
            },
            IdentParent::MemberAccessMemberName(mem_acc) => {
                ctx[*mem_acc].member_name = new_ident.into()
            }
            IdentParent::BuiltinIdent => panic!("builtin ident cannot be changed"),
        }
    }
}
