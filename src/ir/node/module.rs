use id_arena::Id;

use super::{statement::Stmt, NodeRef};

#[derive(Debug)]
pub struct Module<'a> {
    pub stmts: Vec<NodeRef<'a, Stmt<'a>>>,
}
