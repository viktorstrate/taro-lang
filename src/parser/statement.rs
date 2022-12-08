use nom::{
    branch::alt,
    bytes::complete::tag,
    character::streaming::char,
    combinator::{map, opt},
    error::context,
    multi::separated_list0,
    sequence::{preceded, terminated, tuple},
};

use crate::{
    ast::node::{
        statement::{Stmt, StmtValue, VarDecl},
        type_signature::Mutability,
    },
    parser::expression::expression,
};

use super::{
    comment::comment, control_flow::if_branch, enumeration::enumeration, external::external_object,
    function::function_decl, identifier::identifier, spaced, span, structure::structure,
    traits::trait_decl, type_signature::type_signature, ws, Input, Res,
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

    let (i, _) = opt(spaced(tag(";")))(i)?;
    Ok((i, stmt))
}

pub fn single_statement(i: Input<'_>) -> Res<Input<'_>, Stmt<'_>> {
    context(
        "statement",
        map(
            span(alt((
                map(comment, StmtValue::Comment),
                map(variable_decl, StmtValue::VariableDecl),
                map(function_decl, StmtValue::FunctionDecl),
                map(structure, StmtValue::StructDecl),
                map(enumeration, StmtValue::EnumDecl),
                map(external_object, StmtValue::ExternObj),
                stmt_return,
                map(if_branch, StmtValue::IfBranch),
                map(expression, StmtValue::Expression),
                map(trait_decl, StmtValue::TraitDecl),
            ))),
            |(span, value)| Stmt { span, value },
        ),
    )(i)
}

pub fn variable_decl(i: Input<'_>) -> Res<Input<'_>, VarDecl<'_>> {
    // ( val | var ) IDENTIFIER [: TYPE_SIGNATURE] = EXPRESSION

    context(
        "variable declaration",
        map(
            tuple((
                mutability_specifier,
                identifier,
                alt((
                    map(ws, |_| None),
                    opt(preceded(spaced(char(':')), type_signature)),
                )),
                preceded(spaced(char('=')), expression),
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

pub fn mutability_specifier(i: Input<'_>) -> Res<Input<'_>, Mutability> {
    spaced(context(
        "mutability specifier",
        terminated(
            alt((
                map(tag("let"), |_| Mutability::Immutable),
                map(tag("var"), |_| Mutability::Mutable),
            )),
            ws,
        ),
    ))(i)
}

pub fn stmt_return(i: Input<'_>) -> Res<Input<'_>, StmtValue<'_>> {
    context(
        "return",
        map(
            preceded(spaced(tag("return")), expression),
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
            statement(new_input("var name: String = \"John\"")),
            Ok((
                _,
                Stmt {
                    span: Span {
                        line: _,
                        offset: _,
                        fragment: "var name: String = \"John\"",
                        source: _
                    },
                    value: StmtValue::VariableDecl(VarDecl {
                        name: Ident {
                            span: Span {
                                line: _,
                                offset: _,
                                fragment: "name",
                                source: _
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
                                    fragment: "String",
                                    source: _
                                },
                                value: "String"
                            })
                        }),
                        value: Expr {
                            span: Span {
                                line: _,
                                offset: _,
                                fragment: "\"John\"",
                                source: _
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
                        fragment: "let name = true",
                        source: _
                    },
                    value: StmtValue::VariableDecl(VarDecl {
                        name: Ident {
                            span: Span {
                                line: _,
                                offset: _,
                                fragment: "name",
                                source: _
                            },
                            value: "name"
                        },
                        mutability: Mutability::Immutable,
                        type_sig: None,
                        value: Expr {
                            span: Span {
                                line: _,
                                offset: _,
                                fragment: "true",
                                source: _
                            },
                            value: ExprValue::BoolLiteral(true)
                        }
                    })
                }
            ))
        );
    }
}
