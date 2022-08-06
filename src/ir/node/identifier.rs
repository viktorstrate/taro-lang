use id_arena::Id;

use crate::{ir::context::IrCtx, parser::Span};
use std::fmt::Debug;

use super::type_signature::BuiltinType;

pub trait Identifiable<'a> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a>;
}

pub type Ident<'a> = Id<IdentValue<'a>>;

#[derive(Debug)]
pub enum IdentValue<'a> {
    Resolved(ResolvedIdentValue<'a>),
    Unresolved(crate::ast::node::identifier::Ident<'a>),
}

#[derive(Debug)]
pub enum ResolvedIdentValue<'a> {
    Named { def_span: Span<'a>, name: &'a str },
    Anonymous,
    BuiltinType(BuiltinType),
}
