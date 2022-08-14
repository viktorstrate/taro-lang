use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map, opt},
    error::context,
    multi::{separated_list0, separated_list1},
    sequence::{pair, preceded},
};

use crate::ast::node::enumeration::{Enum, EnumValue};

use super::{
    identifier::identifier, surround_brackets, token, type_signature::type_signature, BracketType,
    Input, Res,
};

pub fn enumeration(i: Input<'_>) -> Res<Input<'_>, Enum<'_>> {
    // enum IDENT "{" ENUM_VALUE* "}"

    map(
        pair(
            preceded(token(tag("enum")), identifier),
            surround_brackets(BracketType::Curly, enum_values),
        ),
        move |(name, values)| Enum { name, values },
    )(i)
}

fn enum_values(i: Input<'_>) -> Res<Input<'_>, Vec<EnumValue<'_>>> {
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
        ast::test_utils::{test_ident, test_type_sig},
        parser::new_input,
    };

    use super::*;

    #[test]
    fn test_enum() {
        let enm = enumeration(new_input(
            "enum Test { numbers(Number, Number); string(String) }",
        ))
        .unwrap()
        .1;

        assert_eq!(enm.name, test_ident("Test"));
        assert_eq!(
            enm.values,
            vec![
                EnumValue {
                    name: test_ident("numbers"),
                    items: vec![test_type_sig("Number"), test_type_sig("Number")]
                },
                EnumValue {
                    name: test_ident("string"),
                    items: vec![test_type_sig("String")]
                }
            ]
        );
    }
}
