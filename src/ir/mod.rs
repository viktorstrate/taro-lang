use self::node::module::Module;

pub mod ast_lowering;
pub mod ast_walker;
pub mod context;
pub mod node;
// pub mod test_utils;

#[derive(Debug)]
pub struct IR<'a, 'ctx>(pub Module<'a, 'ctx>);

impl<'a, 'ctx> From<Module<'a, 'ctx>> for IR<'a, 'ctx> {
    fn from(module: Module<'a, 'ctx>) -> Self {
        IR(module)
    }
}
