use crate::{
    ast::node::assignment::Assignment,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::TypeCheckerError;

#[derive(Debug)]
pub enum AssignmentError {}

pub fn check_assignment<'a>(
    _symbols: &mut SymbolTableZipper,
    _asg: &Assignment,
) -> Result<(), TypeCheckerError<'a>> {
    // only assign to:
    // - variable
    // - (nested) struct attribute
    // with properties: mutable, same type

    todo!()
}
