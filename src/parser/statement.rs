use nom::{
    branch::alt,
    bytes::complete::tag,
    character::streaming::char,
    combinator::{cut, map, opt},
    error::context,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::{
    ast::node::{
        statement::{Stmt, VarDecl},
        type_signature::Mutability,
    },
    parser::expression::expression,
};

use super::{
    function::function_decl, identifier::identifier, structure::struct_stmt, token,
    type_signature::type_signature, ws, Res, Span,
};

pub fn statement<'a>(i: Span<'a>) -> Res<Span<'a>, Stmt<'a>> {
    // STMT <<; | \n> STMT>* [;]

    let (i, mut stmts) = separated_list0(alt((tag(";"), tag("\n"))), single_statement)(i)?;

    let (i, stmt) = if stmts.len() == 1 {
        let stmt = stmts.pop().expect("vec should have length 1");
        (i, stmt)
    } else {
        (i, Stmt::Compound(stmts))
    };

    let (i, _) = opt(token(tag(";")))(i)?;
    Ok((i, stmt))
}

pub fn single_statement(i: Span) -> Res<Span, Stmt> {
    context(
        "statement",
        alt((
            variable_decl,
            function_decl,
            struct_stmt,
            stmt_return,
            stmt_expression,
        )),
    )(i)
}

pub fn variable_decl(i: Span) -> Res<Span, Stmt> {
    // let [mut] IDENTIFIER [: TYPE_SIGNATURE] = EXPRESSION

    context(
        "variable declaration",
        map(
            tuple((
                preceded(let_specifier, mut_specifier),
                cut(identifier),
                cut(opt(preceded(token(char(':')), type_signature))),
                cut(preceded(token(char('=')), expression)),
            )),
            |(mutability, name, type_sig, value)| {
                Stmt::VariableDecl(VarDecl {
                    name,
                    mutability,
                    type_sig,
                    value,
                })
            },
        ),
    )(i)
}

pub fn let_specifier(i: Span) -> Res<Span, ()> {
    map(token(tuple((tag("let"), ws))), |_| ())(i)
}

pub fn mut_specifier(i: Span) -> Res<Span, Mutability> {
    context(
        "mut specifier",
        map(opt(token(tuple((tag("mut"), ws)))), |val| {
            val.is_some().into()
        }),
    )(i)
}

pub fn stmt_expression(i: Span) -> Res<Span, Stmt> {
    expression(i).map(|(i, expr)| (i, Stmt::Expression(expr)))
}

pub fn stmt_return(i: Span) -> Res<Span, Stmt> {
    context(
        "return",
        map(
            preceded(token(tag("return")), cut(expression)),
            Stmt::Return,
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::node::{
            expression::Expr,
            identifier::{Ident, IdentValue},
            type_signature::{Mutability, TypeSignature},
        },
        parser::new_span,
    };

    use super::*;

    #[test]
    fn test_stmt() {
        assert_matches!(
            statement(new_span("let mut name: String = \"John\"")),
            Ok((
                _,
                Stmt::VariableDecl(VarDecl {
                    name: Ident {
                        pos: _,
                        value: IdentValue::Named("name")
                    },
                    mutability: Mutability::Mutable,
                    type_sig: Some(TypeSignature::Base(Ident {
                        pos: _,
                        value: IdentValue::Named("String")
                    })),
                    value: Expr::StringLiteral("John")
                })
            ))
        );
    }

    #[test]
    fn test_stmt_type_inferrance() {
        assert_matches!(
            statement(new_span("let name = true")),
            Ok((
                _,
                Stmt::VariableDecl(VarDecl {
                    name: Ident {
                        pos: _,
                        value: IdentValue::Named("name")
                    },
                    mutability: Mutability::Immutable,
                    type_sig: None,
                    value: Expr::BoolLiteral(true)
                })
            ))
        );
    }
}
