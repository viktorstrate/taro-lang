use crate::parser::Span;

use super::{
    comment::Comment,
    control_flow::IfStmt,
    enumeration::Enum,
    expression::Expr,
    external::ExternalObject,
    function::Function,
    identifier::Ident,
    structure::Struct,
    traits::Trait,
    type_signature::{Mutability, TypeSignature},
};

#[derive(Debug, Clone)]
pub struct Stmt<'a> {
    pub span: Span<'a>,
    pub value: StmtValue<'a>,
}

#[derive(Debug, Clone)]
pub enum StmtValue<'a> {
    VariableDecl(VarDecl<'a>),
    FunctionDecl(Function<'a>),
    TraitDecl(Trait<'a>),
    StructDecl(Struct<'a>),
    EnumDecl(Enum<'a>),
    Compound(Vec<Stmt<'a>>),
    Expression(Expr<'a>),
    Return(Expr<'a>),
    Comment(Comment<'a>),
    ExternObj(ExternalObject<'a>),
    IfBranch(IfStmt<'a>),
}

#[derive(Debug, Clone)]
pub struct VarDecl<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub value: Expr<'a>,
}
