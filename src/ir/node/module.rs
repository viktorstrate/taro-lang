use super::{statement::StmtBlock, NodeRef};

#[derive(Debug)]
pub struct Module<'a> {
    pub stmt_block: NodeRef<'a, StmtBlock<'a>>,
}
