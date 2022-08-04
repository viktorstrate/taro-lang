use nom::{
    branch::alt,
    bytes::complete::tag,
    character::streaming::char,
    combinator::{map, opt},
    error::context,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::{
    ast::node::{
        statement::{Stmt, StmtValue, VarDecl},
        type_signature::Mutability,
    },
    parser::expression::expression,
};

use super::{
    enumeration::enumeration, function::function_decl, identifier::identifier, span,
    structure::structure, token, type_signature::type_signature, ws, Input, Res,
};

pub fn statement<'a>(i: Input<'a>) -> Res<Input<'a>, Stmt<'a>> {
    // STMT <<; | \n> STMT>* [;]

    let (i, (span, mut stmts)) = span(separated_list0(
        alt((tag(";"), tag("\n"))),
        single_statement,
    ))(i)?;

    let (i, stmt) = if stmts.len() == 1 {
        let stmt = stmts.pop().expect("vec should have length 1");
        (i, stmt)
    } else {
        (
            i,
            Stmt {
                span,
                value: StmtValue::Compound(stmts),
            },
        )
    };

    let (i, _) = opt(token(tag(";")))(i)?;
    Ok((i, stmt))
}

pub fn single_statement(i: Input<'_>) -> Res<Input<'_>, Stmt<'_>> {
    context(
        "statement",
        map(
            span(alt((
                map(variable_decl, StmtValue::VariableDecl),
                map(function_decl, StmtValue::FunctionDecl),
                map(structure, StmtValue::StructDecl),
                map(enumeration, StmtValue::EnumDecl),
                stmt_return,
                map(expression, StmtValue::Expression),
            ))),
            |(span, value)| Stmt { span, value },
        ),
    )(i)
}

pub fn variable_decl(i: Input<'_>) -> Res<Input<'_>, VarDecl<'_>> {
    // let [mut] IDENTIFIER [: TYPE_SIGNATURE] = EXPRESSION

    context(
        "variable declaration",
        map(
            tuple((
                preceded(let_specifier, mut_specifier),
                identifier,
                opt(preceded(token(char(':')), type_signature)),
                preceded(token(char('=')), expression),
            )),
            |(mutability, name, type_sig, value)| VarDecl {
                name,
                mutability,
                type_sig,
                value,
            },
        ),
    )(i)
}

pub fn let_specifier(i: Input<'_>) -> Res<Input<'_>, ()> {
    map(token(tuple((tag("let"), ws))), |_| ())(i)
}

pub fn mut_specifier(i: Input<'_>) -> Res<Input<'_>, Mutability> {
    context(
        "mut specifier",
        map(opt(token(tuple((tag("mut"), ws)))), |val| {
            val.is_some().into()
        }),
    )(i)
}

pub fn stmt_return(i: Input<'_>) -> Res<Input<'_>, StmtValue<'_>> {
    context(
        "return",
        map(
            preceded(token(tag("return")), expression),
            StmtValue::Return,
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::node::{
            expression::{Expr, ExprValue},
            identifier::Ident,
            type_signature::{TypeSignature, TypeSignatureValue},
        },
        parser::{new_input, Span},
    };

    use super::*;

    #[test]
    fn test_stmt() {
        assert_matches!(
            statement(new_input("let mut name: String = \"John\"")),
            Ok((
                _,
                Stmt {
                    span: Span {
                        line: _,
                        offset: _,
                        fragment: "let mut name: String = \"John\""
                    },
                    value: StmtValue::VariableDecl(VarDecl {
                        name: Ident {
                            span: Span {
                                line: _,
                                offset: _,
                                fragment: "name"
                            },
                            value: "name"
                        },
                        mutability: Mutability::Mutable,
                        type_sig: Some(TypeSignature {
                            span: _,
                            value: TypeSignatureValue::Base(Ident {
                                span: Span {
                                    line: _,
                                    offset: _,
                                    fragment: "String"
                                },
                                value: "String"
                            })
                        }),
                        value: Expr {
                            span: Span {
                                line: _,
                                offset: _,
                                fragment: "\"John\""
                            },
                            value: ExprValue::StringLiteral("John")
                        }
                    })
                }
            ))
        );
    }

    #[test]
    fn test_stmt_type_inferrance() {
        assert_matches!(
            statement(new_input("let name = true")),
            Ok((
                _,
                Stmt {
                    span: Span {
                        line: _,
                        offset: _,
                        fragment: "let name = true"
                    },
                    value: StmtValue::VariableDecl(VarDecl {
                        name: Ident {
                            span: Span {
                                line: _,
                                offset: _,
                                fragment: "name"
                            },
                            value: "name"
                        },
                        mutability: Mutability::Immutable,
                        type_sig: None,
                        value: Expr {
                            span: Span {
                                line: _,
                                offset: _,
                                fragment: "true"
                            },
                            value: ExprValue::BoolLiteral(true)
                        }
                    })
                }
            ))
        );
    }
}
