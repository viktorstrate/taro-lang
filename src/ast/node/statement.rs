use super::{
    enumeration::Enum,
    expression::Expr,
    function::Function,
    identifier::Ident,
    structure::Struct,
    type_signature::{Mutability, TypeSignature},
};

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    VariableDecl(VarDecl<'a>),
    FunctionDecl(Function<'a>),
    StructDecl(Struct<'a>),
    EnumDecl(Enum<'a>),
    Compound(Vec<Stmt<'a>>),
    Expression(Expr<'a>),
    Return(Expr<'a>),
}

#[derive(Debug, Clone)]
pub struct VarDecl<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub value: Expr<'a>,
}
