use id_arena::Id;

use crate::{ir::context::IrCtx, parser::Span};
use std::{fmt::Debug, hash::Hash};

use super::{
    enumeration::{Enum, EnumInit, EnumValue},
    expression::Expr,
    function::{Function, FunctionArg},
    member_access::UnresolvedMemberAccess,
    statement::VarDecl,
    structure::{Struct, StructAccess, StructAttr, StructInit, StructInitValue},
    type_signature::{BuiltinType, TypeSignature},
    NodeRef,
};

pub trait Identifiable<'a> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a>;
}

#[derive(Debug, Clone, Copy)]
pub struct Ident<'a> {
    pub id: Id<IdentValue<'a>>,
    pub parent: IdentParent<'a>,
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
    TypeSigName(TypeSignature<'a>),
    MemberAccessMemberName(NodeRef<'a, UnresolvedMemberAccess<'a>>),
    UnresolvedIdent,
    BuiltinIdent,
}
