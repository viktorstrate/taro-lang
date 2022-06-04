use nom::multi::many0;

use crate::ast::node::{module::Module, statement::Stmt, structure::Struct};

use super::{statement::statement, structure::structure, Res, Span};

pub fn module<'a>(i: Span<'a>) -> Res<Span<'a>, Module<'a>> {
    let mut stmts: Vec<Stmt> = Vec::new();
    let mut structs: Vec<Struct> = Vec::new();

    let mut input = i;

    loop {
        let (i, new_stmt) = statement(input)?;
        let (i, mut new_structs) = many0(structure)(i)?;
        input = i;

        let empty_stmt = if let Stmt::Compound(stmts) = &new_stmt {
            stmts.is_empty()
        } else {
            false
        };

        if empty_stmt && new_structs.is_empty() {
            break;
        }

        if !empty_stmt {
            stmts.push(new_stmt);
        }
        structs.append(&mut new_structs);
    }

    return Ok((input, Module { structs, stmts }));
}

#[cfg(test)]
mod tests {
    use crate::parser::Span;

    use super::module;

    #[test]
    fn test_module() {
        let m = module(Span::new("struct S {} let x = false")).unwrap().1;

        assert_eq!(m.structs.len(), 1);
        assert_eq!(m.stmts.len(), 1);
    }
}
