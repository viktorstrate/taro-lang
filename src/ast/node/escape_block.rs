use super::type_signature::TypeSignature;

#[derive(Debug, Clone)]
pub struct EscapeBlock<'a> {
    pub content: &'a str,
    pub type_sig: Option<TypeSignature<'a>>,
}
