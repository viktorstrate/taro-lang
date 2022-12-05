use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    multi::separated_list0,
    sequence::{pair, preceded},
};

use crate::ast::node::traits::{Trait, TraitFuncAttr};

use super::{
    function::function_signature, identifier::identifier, spaced, span, surround_brackets,
    BracketType, Input, Res,
};

pub fn trait_decl(i: Input<'_>) -> Res<Input<'_>, Trait<'_>> {
    // trait IDENT '{' TRAIT_ATTR+ '}'

    map(
        span(pair(
            preceded(spaced(tag("trait")), identifier),
            surround_brackets(BracketType::Curly, trait_attrs),
        )),
        |(span, (ident, attrs))| Trait {
            name: ident,
            attrs,
            span,
        },
    )(i)
}

pub fn trait_attrs<'a>(i: Input<'a>) -> Res<Input<'a>, Vec<TraitFuncAttr<'a>>> {
    // ATTR <; ATTR>*
    // ATTR <\n ATTR>*

    separated_list0(
        alt((tag(";"), tag("\n"))),
        map(function_signature, |(ident, func_args, ret_type, span)| {
            TraitFuncAttr {
                name: ident,
                args: func_args,
                return_type: ret_type,
                span,
            }
        }),
    )(i)
}

#[cfg(test)]
mod tests {

    use crate::{ast::test_utils::test_ident, parser::new_input};

    use super::*;

    #[test]
    fn test_trait() {
        let tr = trait_decl(new_input(
            "trait Additive { func add(a: Number, b: Number) -> Number }",
        ))
        .unwrap()
        .1;

        assert_eq!(tr.name, test_ident("Additive"));
        assert_eq!(tr.attrs.len(), 1);
    }
}
