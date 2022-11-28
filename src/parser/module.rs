use nom::{combinator::eof, sequence::terminated};

use crate::ast::node::module::Module;

use super::{spaced, statement::statement, Input, ParserError};

pub fn module<'a>(i: Input<'a>) -> Result<Module<'a>, ParserError<'a>> {
    match terminated(spaced(statement), eof)(i) {
        Ok((_, stmt)) => Ok(Module { stmt }),
        Err(err) => match err {
            nom::Err::Incomplete(_size) => Err(ParserError::EarlyTermination()),
            nom::Err::Error(e) | nom::Err::Failure(e) => Err(ParserError::NomErr(e)),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{ast::node::statement::StmtValue, parser::new_input};

    use super::module;

    #[test]
    fn test_module() {
        let m = module(new_input("struct S {}\nlet x = false")).unwrap();

        match m.stmt.value {
            StmtValue::Compound(cmp) => {
                assert_eq!(cmp.len(), 2)
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_invalid_end() {
        let m = module(new_input("let x = 2 INVALID"));
        assert_matches!(m, Err(_));
    }
}
