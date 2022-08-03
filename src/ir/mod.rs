use self::node::module::Module;

pub mod ast_walker;
pub mod context;
pub mod node;
// pub mod test_utils;

#[derive(Debug)]
pub struct AST<'a, 'ctx>(Module<'a, 'ctx>);

impl<'a, 'ctx> AST<'a, 'ctx> {
    pub fn inner_module(&self) -> &Module<'a, 'ctx> {
        &self.0
    }
}

impl<'a, 'ctx> From<Module<'a, 'ctx>> for AST<'a, 'ctx> {
    fn from(module: Module<'a, 'ctx>) -> Self {
        AST(module)
    }
}
