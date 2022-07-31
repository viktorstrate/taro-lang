use crate::ir::node::{module::Module, statement::Stmt};

use super::{statement::statement, Res, Span};

pub fn module<'a>(i: Span<'a>) -> Res<Span<'a>, Module<'a>> {
    let mut stmts: Vec<Stmt> = Vec::new();

    let mut input = i;

    loop {
        let (i, new_stmt) = statement(input)?;
        input = i;

        let empty_stmt = if let Stmt::Compound(stmts) = &new_stmt {
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
    use crate::parser::new_span;

    use super::module;

    #[test]
    fn test_module() {
        let m = module(new_span("struct S {} let x = false")).unwrap().1;

        assert_eq!(m.stmts.len(), 2);
    }
}
