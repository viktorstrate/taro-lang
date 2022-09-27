use nom::{
    branch::alt,
    bytes::complete::tag,
    character::streaming::char,
    combinator::{map, opt},
    error::context,
    multi::separated_list0,
    sequence::{pair, preceded, tuple},
};

use crate::ast::node::structure::{Struct, StructAttr, StructInit, StructInitValue};

use super::{
    expression::expression, identifier::identifier, spaced, span, statement::mutability_specifier,
    surround_brackets, type_signature::type_signature, ws, BracketType, Input, Res,
};

pub fn structure<'a>(i: Input<'a>) -> Res<Input<'a>, Struct<'a>> {
    // "struct" IDENT { STRUCT_ATTRS }

    context(
        "structure declaration",
        map(
            tuple((
                preceded(spaced(tuple((tag("struct"), ws))), identifier),
                surround_brackets(BracketType::Curly, struct_attrs),
            )),
            move |(name, attrs)| Struct { name, attrs },
        ),
    )(i)
}

pub fn struct_attrs<'a>(i: Input<'a>) -> Res<Input<'a>, Vec<StructAttr<'a>>> {
    // ATTR <; ATTR>*
    // ATTR <\n ATTR>*

    let struct_attr = move |i: Input<'a>| -> Res<Input<'a>, StructAttr<'a>> {
        // (val | var) IDENT [ : TYPE_SIG ] [ = EXPR ]

        map(
            context(
                "structure attribute",
                span(tuple((
                    mutability_specifier,
                    context("attribute identifier", identifier),
                    context(
                        "attribute type signature",
                        opt(preceded(spaced(char(':')), type_signature)),
                    ),
                    opt(preceded(
                        spaced(char('=')),
                        context("attribute default value", expression),
                    )),
                ))),
            ),
            |(span, (mutability, name, type_sig, default_value))| StructAttr {
                name,
                mutability,
                type_sig: type_sig,
                default_value,
                span: span,
            },
        )(i)
    };

    separated_list0(
        alt((tag(";"), tag("\n"))),
        context("struct attribute", struct_attr),
    )(i)
}

pub fn struct_init_expr(i: Input<'_>) -> Res<Input<'_>, StructInit<'_>> {
    // IDENT "{" <IDENT: EXPR> , ... "}"

    let struct_init_value = map(
        span(tuple((identifier, preceded(spaced(tag(":")), expression)))),
        |(span, (name, value))| StructInitValue { name, value, span },
    );

    map(
        span(pair(
            identifier,
            surround_brackets(
                BracketType::Curly,
                separated_list0(spaced(tag(",")), struct_init_value),
            ),
        )),
        |(span, (struct_name, values))| StructInit {
            struct_name,
            values,
            span,
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::{
            node::{
                expression::{Expr, ExprValue},
                identifier::Ident,
                member_access::MemberAccess,
                type_signature::Mutability,
            },
            test_utils::{test_ident, test_type_sig},
        },
        parser::new_input,
    };

    use super::*;

    #[test]
    fn test_struct() {
        let st = structure(new_input("struct Example { let attr: String }"))
            .unwrap()
            .1;

        assert_eq!(st.name, test_ident("Example"));
        assert_eq!(st.attrs.len(), 1);
        assert_eq!(st.attrs[0].name, test_ident("attr"));
        assert_eq!(st.attrs[0].mutability, Mutability::Immutable);
        assert_eq!(st.attrs[0].type_sig, Some(test_type_sig("String")));
        assert!(st.attrs[0].default_value.is_none());
        assert_eq!(st.attrs[0].span.fragment, "let attr: String");
    }

    #[test]
    fn test_struct_init() {
        let struct_init = expression(new_input("StructName { attr: true }"))
            .unwrap()
            .1
            .value;

        match struct_init {
            ExprValue::StructInit(StructInit {
                struct_name: name,
                values,
                span: _,
            }) => {
                assert_eq!(name, test_ident("StructName"));
                assert_eq!(values.len(), 1);
                assert_eq!(values[0].name, test_ident("attr"));
                assert_matches!(
                    values[0].value,
                    Expr {
                        span: _,
                        value: ExprValue::BoolLiteral(true)
                    }
                );
                assert_eq!(values[0].span.fragment, "attr: true")
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_struct_access_simple() {
        let struct_access = expression(new_input("struct_name.attribute"))
            .unwrap()
            .1
            .value;

        match struct_access {
            ExprValue::MemberAccess(mem_acc) => match *mem_acc {
                MemberAccess {
                    object,
                    member_name,
                    items,
                    span: _,
                } => {
                    assert_matches!(
                        object,
                        Some(Expr {
                            span: _,
                            value: ExprValue::Identifier(Ident {
                                span: _,
                                value: "struct_name"
                            })
                        })
                    );
                    assert_eq!(member_name, test_ident("attribute"));
                    assert!(items.is_none());
                }
            },
            _ => assert!(false),
        }
    }
}
