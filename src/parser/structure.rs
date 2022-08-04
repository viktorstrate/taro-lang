use nom::{
    branch::alt,
    bytes::complete::tag,
    character::streaming::char,
    combinator::{map, opt},
    error::context,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::ast::node::structure::{Struct, StructAttr, StructInit, StructInitValue};

use super::{
    expression::expression,
    identifier::identifier,
    span,
    statement::{let_specifier, mut_specifier},
    surround_brackets, token,
    type_signature::type_signature,
    ws, BracketType, Input, Res,
};

pub fn structure<'a>(i: Input<'a>) -> Res<Input<'a>, Struct<'a>> {
    // "struct" IDENT { STRUCT_ATTRS }

    context(
        "structure declaration",
        map(
            tuple((
                preceded(token(tuple((tag("struct"), ws))), identifier),
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
        // let [mut] IDENT [ : TYPE_SIG ] [ = EXPR ]

        map(
            context(
                "structure attribute",
                span(tuple((
                    preceded(let_specifier, mut_specifier),
                    context("attribute identifier", identifier),
                    context(
                        "attribute type signature",
                        opt(preceded(token(char(':')), type_signature)),
                    ),
                    opt(preceded(
                        token(char('=')),
                        context("attribute default value", expression),
                    )),
                ))),
            ),
            |(span, (mutability, name, type_sig, default_value))| StructAttr {
                name,
                mutability,
                type_sig,
                default_value,
                span,
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

    let (i, struct_name) = identifier(i)?;

    let (i, values) = surround_brackets(
        BracketType::Curly,
        separated_list0(
            token(tag(",")),
            map(
                span(tuple((identifier, preceded(token(tag(":")), expression)))),
                |(span, (name, value))| StructInitValue { name, value, span },
            ),
        ),
    )(i)?;

    Ok((
        i,
        StructInit {
            struct_name,
            values,
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::{
            node::{
                expression::{Expr, ExprValue},
                structure::StructAccess,
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
            ExprValue::StructAccess(StructAccess {
                struct_expr: _,
                attr_name,
            }) => {
                assert_eq!(attr_name, test_ident("attribute"));
            }
            _ => assert!(false),
        }
    }
}
