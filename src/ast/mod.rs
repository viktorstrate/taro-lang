use self::node::module::Module;

pub mod node;
pub mod test_utils;

#[derive(Debug)]
pub struct AST<'a>(pub Module<'a>);

impl<'a> From<Module<'a>> for AST<'a> {
    fn from(module: Module<'a>) -> Self {
        AST(module)
    }
}
