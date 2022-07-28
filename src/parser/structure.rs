use nom::{
    branch::alt,
    bytes::complete::tag,
    character::streaming::char,
    combinator::{cut, map, opt},
    error::context,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::ast::node::{
    expression::Expr,
    identifier::Ident,
    statement::Stmt,
    structure::{Struct, StructAttr, StructInit, StructInitValue},
};

use super::{
    expression::expression,
    identifier::identifier,
    statement::{let_specifier, mut_specifier},
    surround_brackets, token,
    type_signature::type_signature,
    ws, BracketType, Res, Span,
};

pub fn structure<'a>(mut i: Span<'a>) -> Res<Span<'a>, Struct<'a>> {
    // "struct" IDENT { STRUCT_ATTRS }

    let ref_id = i.extra.ref_gen.make_ref();

    context(
        "struct",
        map(
            tuple((
                preceded(token(tuple((tag("struct"), ws))), identifier),
                cut(surround_brackets(BracketType::Curly, struct_attrs)),
            )),
            move |(name, attrs)| Struct {
                name,
                attrs,
                ref_id,
            },
        ),
    )(i)
}

pub fn struct_attrs<'a>(i: Span<'a>) -> Res<Span, Vec<StructAttr<'a>>> {
    // ATTR <; ATTR>*
    // ATTR <\n ATTR>*

    let struct_attr = move |i: Span<'a>| -> Res<Span, StructAttr<'a>> {
        // let [mut] IDENT [ : TYPE_SIG ] [ = EXPR ]

        map(
            tuple((
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
            )),
            |(mutability, name, type_sig, default_value)| StructAttr {
                name,
                mutability,
                type_sig,
                default_value,
            },
        )(i)
    };

    separated_list0(
        alt((tag(";"), tag("\n"))),
        context("struct attribute", struct_attr),
    )(i)
}

pub fn struct_stmt(i: Span) -> Res<Span, Stmt> {
    map(structure, Stmt::StructDecl)(i)
}

pub fn struct_init_expr(i: Span) -> Res<Span, Expr> {
    // IDENT "{" <IDENT: EXPR> , ... "}"

    let (i, struct_name) = identifier(i)?;

    let (mut i, values) = surround_brackets(
        BracketType::Curly,
        separated_list0(
            token(tag(",")),
            map(
                tuple((identifier, preceded(token(tag(":")), expression))),
                |(name, value)| StructInitValue { name, value },
            ),
        ),
    )(i)?;

    let ref_id = i.extra.ref_gen.make_ref();

    Ok((
        i,
        Expr::StructInit(StructInit {
            struct_name,
            scope_name: Ident::new_anon(ref_id),
            values,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::node::{identifier::Ident, structure::StructAccess, type_signature::Mutability},
        parser::new_span,
        symbols::builtin_types::BuiltinType,
    };

    use super::*;

    #[test]
    fn test_struct() {
        let st = structure(new_span("struct Example { let attr: String }"))
            .unwrap()
            .1;

        assert_eq!(st.name, Ident::new_unplaced("Example"));
        assert_eq!(st.attrs.len(), 1);
        assert_eq!(st.attrs[0].name, Ident::new_unplaced("attr"));
        assert_eq!(st.attrs[0].mutability, Mutability::Immutable);
        assert_eq!(st.attrs[0].type_sig, Some(BuiltinType::String.type_sig()));
        assert!(st.attrs[0].default_value.is_none());
    }

    #[test]
    fn test_struct_init() {
        let struct_init = expression(new_span("StructName { attr: true }")).unwrap().1;

        match struct_init {
            Expr::StructInit(StructInit {
                struct_name: name,
                scope_name: _,
                values,
            }) => {
                assert_eq!(name, Ident::new_unplaced("StructName"));
                assert_eq!(values.len(), 1);
                assert_eq!(values[0].name, Ident::new_unplaced("attr"));
                assert_matches!(values[0].value, Expr::BoolLiteral(true));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_struct_access_simple() {
        let struct_access = expression(new_span("struct_name.attribute")).unwrap().1;

        match struct_access {
            Expr::StructAccess(StructAccess {
                struct_expr: _,
                attr_name,
            }) => {
                assert_eq!(attr_name, Ident::new_unplaced("attribute"));
            }
            _ => assert!(false),
        }
    }
}
