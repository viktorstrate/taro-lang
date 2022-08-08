use id_arena::Id;

use crate::{ir::context::IrCtx, parser::Span};
use std::fmt::Debug;

use super::type_signature::BuiltinType;

pub trait Identifiable<'a> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ident<'a> {
    id: Id<IdentValue<'a>>,
}

impl<'a> Into<Id<IdentValue<'a>>> for Ident<'a> {
    fn into(self) -> Id<IdentValue<'a>> {
        self.id
    }
}

impl<'a> From<Id<IdentValue<'a>>> for Ident<'a> {
    fn from(id: Id<IdentValue<'a>>) -> Self {
        Self { id }
    }
}

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
}
