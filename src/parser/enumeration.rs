use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    error::context,
    multi::{many1, separated_list0, separated_list1},
    sequence::{pair, preceded},
};

use crate::ast::node::enumeration::{Enum, EnumValue};

use super::{
    identifier::identifier, surround_brackets, token, type_signature::type_signature, BracketType,
    Res, Span,
};

pub fn enumeration(i: Span) -> Res<Span, Enum> {
    // enum IDENT "{" ENUM_VALUE* "}"

    map(
        pair(
            preceded(token(tag("enum")), identifier),
            surround_brackets(BracketType::Curly, enum_values),
        ),
        |(name, values)| Enum { name, values },
    )(i)
}

fn enum_values(i: Span) -> Res<Span, Vec<EnumValue>> {
    // IDENT [ "(" TYPE_SIG+ ")" ]
    // let (i, name) = identifier(i)?;
    // let (i, items) = opt(surround_brackets(BracketType::Round, many1(type_signature)))(i)?;

    let enum_value = map(
        pair(
            identifier,
            opt(surround_brackets(
                BracketType::Round,
                separated_list1(tag(","), type_signature),
            )),
        ),
        |(name, items)| EnumValue {
            name,
            items: items.unwrap_or_default(),
        },
    );

    separated_list0(
        alt((tag(";"), tag("\n"))),
        context("enum value", enum_value),
    )(i)
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::node::identifier::Ident, parser::new_span, symbols::builtin_types::BuiltinType,
    };

    use super::*;

    #[test]
    fn test_enum() {
        let enm = enumeration(new_span(
            "enum Test { numbers(Number, Number); string(String) }",
        ))
        .unwrap()
        .1;

        assert_eq!(enm.name, Ident::new_unplaced("Test"));
        assert_eq!(
            enm.values,
            vec![
                EnumValue {
                    name: Ident::new_unplaced("numbers"),
                    items: vec![
                        BuiltinType::Number.type_sig(),
                        BuiltinType::Number.type_sig()
                    ]
                },
                EnumValue {
                    name: Ident::new_unplaced("string"),
                    items: vec![BuiltinType::String.type_sig()]
                }
            ]
        );
    }
}
