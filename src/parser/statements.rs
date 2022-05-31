use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::{alphanumeric0, satisfy},
        streaming::char,
    },
    combinator::{opt, recognize},
    multi::separated_list1,
    sequence::{pair, preceded},
};

use crate::{
    ast::{Ident, Stmt, TypeSignature, VarDecl},
    parser::expressions::expression,
};

use super::{token, Res};

pub fn statement(i: &str) -> Res<&str, Stmt> {
    // STMT <; STMT>*
    // STMT <\n STMT>*

    let stmt_separator = alt((tag(";"), tag("\n")));
    let (i, mut stmts) = separated_list1(stmt_separator, single_statement)(i)?;

    if stmts.len() == 1 {
        let stmt = stmts.pop().expect("vec should have length 1");
        Ok((i, stmt))
    } else {
        Ok((i, Stmt::Compound(stmts)))
    }
}

pub fn single_statement(i: &str) -> Res<&str, Stmt> {
    declaration_variable(i)
}

pub fn declaration_variable(i: &str) -> Res<&str, Stmt> {
    // let IDENTIFIER [: TYPE_SIGNATURE] = EXPRESSION

    let (i, _) = token(tag("let"))(i)?;
    let (i, is_mut) = opt(token(tag("mut")))(i)?;
    let (i, name) = identifier(i)?;

    let (i, type_sig) = opt(preceded(token(char(':')), type_signature))(i)?;
    let (i, value) = preceded(token(char('=')), expression)(i)?;

    let var_decl = VarDecl {
        name,
        mutability: is_mut.is_some().into(),
        type_sig,
        value,
    };

    Ok((i, Stmt::VarDecl(var_decl)))
}

pub fn identifier(i: &str) -> Res<&str, Ident> {
    token(recognize(pair(
        satisfy(|c| c.is_alphabetic()),
        alphanumeric0,
    )))(i)
    .map(|(i, val)| (i, Ident::new(val)))
}

pub fn type_signature(i: &str) -> Res<&str, TypeSignature> {
    type_sig_base(i)
}

fn type_sig_base(i: &str) -> Res<&str, TypeSignature> {
    return identifier(i).map(|(i, ident)| (i, TypeSignature::Base(ident)));
}

#[cfg(test)]
mod tests {
    use crate::ast::{Expr, Mutability};

    use super::*;

    #[test]
    fn test_stmt() {
        assert_eq!(
            statement("let mut name: Number = 23"),
            Ok((
                "",
                Stmt::VarDecl(VarDecl {
                    name: "name".into(),
                    mutability: Mutability::Mutable,
                    type_sig: Some(TypeSignature::Base("Number".into())),
                    value: Expr::NumberLiteral(23.0)
                })
            ))
        );
    }

    #[test]
    fn test_stmt_type_inferrance() {
        assert_eq!(
            statement("let name = 23"),
            Ok((
                "",
                Stmt::VarDecl(VarDecl {
                    name: "name".into(),
                    mutability: Mutability::Immutable,
                    type_sig: None,
                    value: Expr::NumberLiteral(23.0)
                })
            ))
        );
    }
}
