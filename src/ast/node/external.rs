use crate::parser::Span;

use super::{identifier::Ident, type_signature::TypeSignature};

#[derive(Debug, Clone)]
pub struct ExternalObject<'a> {
    pub ident: Ident<'a>,
    pub type_sig: TypeSignature<'a>,
    pub span: Span<'a>,
}
