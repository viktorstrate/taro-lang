use super::statement::Stmt;

#[derive(Debug, Clone)]
pub struct Module<'a> {
    pub stmt: Stmt<'a>,
}
