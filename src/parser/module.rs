use crate::ast::node::module::Module;

use super::{statement::statement, Input, Res};

pub fn module<'a>(i: Input<'a>) -> Res<Input<'a>, Module<'a>> {
    let (i, stmt) = statement(i)?;
    return Ok((i, Module { stmt }));
}

#[cfg(test)]
mod tests {
    use crate::{ast::node::statement::StmtValue, parser::new_input};

    use super::module;

    #[test]
    fn test_module() {
        let m = module(new_input("struct S {}\nlet x = false")).unwrap().1;

        match m.stmt.value {
            StmtValue::Compound(cmp) => {
                assert_eq!(cmp.len(), 2)
            }
            _ => assert!(false),
        }
    }
}
