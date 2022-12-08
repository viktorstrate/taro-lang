use crate::parser::Span;

use super::{function::FunctionArg, identifier::Ident, type_signature::TypeSignature};

#[derive(Debug, Clone)]
pub struct Trait<'a> {
    pub name: Ident<'a>,
    pub attrs: Vec<TraitFuncAttr<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct TraitFuncAttr<'a> {
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub span: Span<'a>,
}
