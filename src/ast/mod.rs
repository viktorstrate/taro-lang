use self::nodes::statements::Stmt;

pub mod ast_walker;
pub mod nodes;

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
