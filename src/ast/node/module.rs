use super::statement::Stmt;

#[derive(Debug, Clone)]
pub struct Module<'a> {
    pub stmts: Vec<Stmt<'a>>,
}
