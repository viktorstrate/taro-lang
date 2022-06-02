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

use super::{token, Res, Span};

pub fn statement(i: Span) -> Res<Span, Stmt> {
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

pub fn single_statement(i: Span) -> Res<Span, Stmt> {
    declaration_variable(i)
}

pub fn declaration_variable(i: Span) -> Res<Span, Stmt> {
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

pub fn identifier(i: Span) -> Res<Span, Ident> {
    token(recognize(pair(
        satisfy(|c| c.is_alphabetic()),
        alphanumeric0,
    )))(i)
    .map(|(i, val)| (i, Ident::new(i, &val)))
}

pub fn type_signature(i: Span) -> Res<Span, TypeSignature> {
    type_sig_base(i)
}

fn type_sig_base(i: Span) -> Res<Span, TypeSignature> {
    return identifier(i).map(|(i, ident)| (i, TypeSignature::Base(ident)));
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ast::{Expr, Mutability};

    use super::*;

    #[test]
    fn test_stmt() {
        assert_matches!(
            statement(Span::new("let mut name: Number = 23")),
            Ok((
                _,
                Stmt::VarDecl(VarDecl {
                    name: Ident {
                        pos: _,
                        value: "name"
                    },
                    mutability: Mutability::Mutable,
                    type_sig: Some(TypeSignature::Base(Ident {
                        pos: _,
                        value: "Number"
                    })),
                    value: Expr::NumberLiteral(23.0)
                })
            ))
        );
    }

    #[test]
    fn test_stmt_type_inferrance() {
        assert_matches!(
            statement(Span::new("let name = 23")),
            Ok((
                _,
                Stmt::VarDecl(VarDecl {
                    name: Ident {
                        pos: _,
                        value: "name"
                    },
                    mutability: Mutability::Immutable,
                    type_sig: None,
                    value: Expr::NumberLiteral(23.0)
                })
            ))
        );
    }
}
