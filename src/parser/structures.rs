use nom::{
    branch::alt,
    bytes::complete::tag,
    character::streaming::char,
    combinator::opt,
    multi::separated_list1,
    sequence::{preceded, tuple},
};

use crate::ast::nodes::structures::{Struct, StructAttr};

use super::{
    expressions::expression,
    statements::{identifier, type_signature},
    surround_brackets, token, ws, BracketType, Res, Span,
};

pub fn structure<'a>(i: Span<'a>) -> Res<Span<'a>, Struct<'a>> {
    // struct IDENT { STRUCT_ATTRS }

    let (i, _) = token(tuple((tag("struct"), ws)))(i)?;

    let (i, ident) = identifier(i)?;

    let (i, attrs) = surround_brackets(BracketType::Curly, struct_attrs)(i)?;

    Ok((i, Struct { name: ident, attrs }))
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

    separated_list1(alt((tag(";"), tag("\n"))), struct_attr)(i)
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::nodes::type_signature::{BuiltinType, Mutability},
        parser::Span,
    };

    use super::*;

    #[test]
    fn test_struct() {
        let st = structure(Span::new("struct Example { let attr: String }"))
            .unwrap()
            .1;

        assert_eq!(st.name.value, "Example");
        assert_eq!(st.attrs.len(), 1);
        assert_eq!(st.attrs[0].name.value, "attr");
        assert_eq!(st.attrs[0].mutability, Mutability::Immutable);
        assert_eq!(st.attrs[0].type_sig, Some(BuiltinType::String.into()));
        assert_eq!(st.attrs[0].default_value, None);
    }
}
