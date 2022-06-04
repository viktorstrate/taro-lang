use super::{statement::Stmt, structure::Struct};

#[derive(Debug)]
pub struct Module<'a> {
    pub structs: Vec<Struct<'a>>,
    pub stmts: Vec<Stmt<'a>>,
}
