use self::node::module::Module;

pub mod ast_lowering;
pub mod context;
pub mod ir_walker;
pub mod late_init;
pub mod node;
pub mod test_utils;

#[derive(Debug)]
pub struct IR<'a>(pub Module<'a>);

impl<'a> From<Module<'a>> for IR<'a> {
    fn from(module: Module<'a>) -> Self {
        IR(module)
    }
}
