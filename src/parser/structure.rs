use nom::{
    branch::alt,
    bytes::complete::tag,
    character::streaming::char,
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::ast::node::{
    statement::Stmt,
    structure::{Struct, StructAttr},
};

use super::{
    expression::expression, identifier::identifier, statement::type_signature, surround_brackets,
    token, ws, BracketType, Res, Span,
};

pub fn structure<'a>(i: Span<'a>) -> Res<Span<'a>, Struct<'a>> {
    // struct IDENT { STRUCT_ATTRS }

    let (i, _) = token(tuple((tag("struct"), ws)))(i)?;

    let (i, ident) = identifier(i)?;

    let (mut i, attrs) = surround_brackets(BracketType::Curly, struct_attrs)(i)?;

    let ref_id = i.extra.ref_gen.make_ref();

    Ok((
        i,
        Struct {
            name: ident,
            attrs,
            ref_id,
        },
    ))
}

pub fn struct_attrs<'a>(i: Span<'a>) -> Res<Span, Vec<StructAttr<'a>>> {
    // ATTR <; ATTR>*
    // ATTR <\n ATTR>*

    let struct_attr = move |i: Span<'a>| -> Res<Span, StructAttr<'a>> {
        // let [mut] IDENT [ = EXPR ]

        let (i, _) = token(tuple((tag("let"), ws)))(i)?;
        let (i, is_mut) = opt(token(tuple((tag("mut"), ws))))(i)?;
        let (i, name) = identifier(i)?;

        let (i, type_sig) = opt(preceded(token(char(':')), type_signature))(i)?;
        let (i, default_value) = opt(preceded(token(char('=')), expression))(i)?;

        Ok((
            i,
            StructAttr {
                name,
                mutability: is_mut.is_some().into(),
                type_sig,
                default_value,
            },
        ))
    };

    separated_list0(alt((tag(";"), tag("\n"))), struct_attr)(i)
}

pub fn struct_stmt(i: Span) -> Res<Span, Stmt> {
    map(structure, Stmt::StructDecl)(i)
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::node::{
            identifier::Ident,
            type_signature::{BuiltinType, Mutability},
        },
        parser::new_span,
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
        assert_eq!(st.attrs[0].type_sig, Some(BuiltinType::String.into()));
        assert!(st.attrs[0].default_value.is_none());
    }
}
