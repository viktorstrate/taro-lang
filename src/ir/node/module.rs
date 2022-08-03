use super::statement::Stmt;

#[derive(Debug)]
pub struct Module<'a, 'ctx> {
    pub stmts: Vec<&'ctx mut Stmt<'a, 'ctx>>,
}
