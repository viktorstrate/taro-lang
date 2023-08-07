use nom::{bytes::complete::tag, combinator::map, multi::separated_list0};

use crate::ast::node::{generics::GenericsDecl, identifier::Ident};

use super::{identifier::identifier, spaced, span, surround_brackets, BracketType, Input, Res};

pub fn generics_decl(i: Input<'_>) -> Res<Input<'_>, GenericsDecl<'_>> {
    // "<" GENERIC_TYPE,+ ">"

    map(
        span(surround_brackets(
            BracketType::Diamond,
            separated_list0(spaced(tag(",")), generic_type),
        )),
        |(span, generics)| GenericsDecl { span, generics },
    )(i)
}

pub fn generic_type(i: Input<'_>) -> Res<Input<'_>, Ident<'_>> {
    identifier(i)
}
