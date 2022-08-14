use crate::ast::node::{
    module::Module,
    statement::{Stmt, StmtValue},
};

use super::{statement::statement, Input, Res};

pub fn module<'a>(i: Input<'a>) -> Res<Input<'a>, Module<'a>> {
    let mut stmts: Vec<Stmt<'_>> = Vec::new();

    let mut input = i;

    loop {
        let (i, new_stmt) = statement(input)?;
        input = i;

        let empty_stmt = if let StmtValue::Compound(stmts) = &new_stmt.value {
            stmts.is_empty()
        } else {
            false
        };

        if empty_stmt {
            break;
        }

        stmts.push(new_stmt);
    }

    return Ok((input, Module { stmts }));
}

#[cfg(test)]
mod tests {
    use crate::parser::new_input;

    use super::module;

    #[test]
    fn test_module() {
        let m = module(new_input("struct S {} let x = false")).unwrap().1;

        assert_eq!(m.stmts.len(), 2);
    }
}
