use nom::multi::many0;

use crate::ast::nodes::{module::Module, statements::Stmt, structures::Struct};

use super::{statements::statement, structures::structure, Res, Span};

pub fn module<'a>(i: Span<'a>) -> Res<Span<'a>, Module<'a>> {
    let mut stmts: Vec<Stmt> = Vec::new();
    let mut structs: Vec<Struct> = Vec::new();

    let mut input = i;

    loop {
        let (i, mut new_stmts) = many0(statement)(input)?;
        let (i, mut new_structs) = many0(structure)(i)?;
        input = i;

        if new_stmts.is_empty() && new_structs.is_empty() {
            break;
        }

        stmts.append(&mut new_stmts);
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
