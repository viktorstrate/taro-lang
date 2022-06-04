use self::node::module::Module;

pub mod ast_walker;
pub mod node;

#[derive(Debug)]
pub struct AST<'a>(Module<'a>);

impl<'a> AST<'a> {
    pub fn inner_module(&self) -> &Module<'a> {
        &self.0
    }
}

impl<'a> From<Module<'a>> for AST<'a> {
    fn from(module: Module<'a>) -> Self {
        AST(module)
    }
}
