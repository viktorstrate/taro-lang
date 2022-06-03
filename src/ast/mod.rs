use crate::parser::Span;
use std::hash::Hash;

pub mod ast_walker;

#[derive(Debug)]
pub struct AST<'a>(Stmt<'a>);

impl<'a> AST<'a> {
    pub fn inner_stmt(&self) -> &Stmt<'a> {
        &self.0
    }
}

impl<'a> From<Stmt<'a>> for AST<'a> {
    fn from(stmt: Stmt<'a>) -> Self {
        AST(stmt)
    }
}

#[derive(PartialEq, Debug)]
pub enum Stmt<'a> {
    VarDecl(VarDecl<'a>),
    Compound(Vec<Stmt<'a>>),
}

pub trait Identifiable<'a> {
    fn name(&self) -> &Ident<'a>;
}

#[derive(PartialEq, Debug, Clone)]
pub struct VarDecl<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub value: Expr<'a>,
}

impl<'a> Identifiable<'a> for VarDecl<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
}

impl<'a> Expr<'a> {
    pub fn value_type(&self) -> TypeSignature<'a> {
        match self {
            &Self::StringLiteral(..) => BuiltinType::String.into(),
            &Self::NumberLiteral(..) => BuiltinType::Number.into(),
            &Self::BoolLiteral(..) => BuiltinType::Bool.into(),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Mutability {
    Mutable,
    Immutable,
}

impl From<bool> for Mutability {
    fn from(val: bool) -> Self {
        if val {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }
    }
}

impl Into<bool> for Mutability {
    fn into(self) -> bool {
        self == Mutability::Mutable
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum TypeSignature<'a> {
    Base(Ident<'a>),
    Function(
        Ident<'a>,
        Box<Vec<TypeSignature<'a>>>,
        Box<TypeSignature<'a>>,
    ),
    Reference(Box<TypeSignature<'a>>),
    // GenericBase(Ident<'a>, Box<Vec<TypeSignatureValue<'a>>>),
}

#[derive(Debug)]
pub enum BuiltinType {
    String,
    Number,
    Bool,
}

impl Into<TypeSignature<'static>> for BuiltinType {
    fn into(self) -> TypeSignature<'static> {
        let value = match self {
            BuiltinType::String => "String",
            BuiltinType::Number => "Number",
            BuiltinType::Bool => "Bool",
        };
        TypeSignature::Base(Ident::new_unplaced(value))
    }
}

#[derive(Debug, Clone)]
pub struct Ident<'a> {
    pub pos: Span<'a>,
    pub value: &'a str,
}

impl<'a> Ident<'a> {
    pub fn new(pos: Span<'a>, value: &'a str) -> Self {
        Ident {
            pos: pos,
            value: value,
        }
    }

    fn new_unplaced(value: &'a str) -> Self {
        Ident {
            pos: Span::new(""),
            value,
        }
    }
}

impl PartialEq for Ident<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Ident<'_> {}

impl Hash for Ident<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
