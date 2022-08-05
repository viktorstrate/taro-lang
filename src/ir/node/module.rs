use id_arena::Id;

use super::statement::Stmt;

#[derive(Debug)]
pub struct Module<'a> {
    pub stmts: Vec<Id<Stmt<'a>>>,
}
