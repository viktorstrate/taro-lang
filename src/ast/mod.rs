pub mod ast_walker;

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
    fn value_type(&self) -> TypeSignature {
        match self {
            &Self::StringLiteral(..) => TypeSignature::BUILTIN_STRING,
            &Self::NumberLiteral(..) => TypeSignature::BUILTIN_NUMBER,
            &Self::BoolLiteral(..) => TypeSignature::BUILTIN_BOOL,
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
    // GenericBase(&'a str, Box<Vec<TypeSignatureValue<'a>>>),
    // Array(Box<TypeSignatureValue<'a>>),
    // Optional(Box<TypeSignatureValue<'a>>)
}

impl TypeSignature<'static> {
    const BUILTIN_STRING: Self = TypeSignature::Base(Ident("String"));
    const BUILTIN_NUMBER: Self = TypeSignature::Base(Ident("Number"));
    const BUILTIN_BOOL: Self = TypeSignature::Base(Ident("Bool"));
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Ident<'a>(&'a str);

impl<'a> Ident<'a> {
    pub fn new(value: &'a str) -> Self {
        Ident(value)
    }
}

impl<'a> From<&'a str> for Ident<'a> {
    fn from(raw: &'a str) -> Self {
        Ident(raw)
    }
}

impl<'a> Ident<'a> {
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}
