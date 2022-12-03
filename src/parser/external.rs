use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::map,
    sequence::{pair, preceded, tuple},
};

use crate::ast::node::external::ExternalObject;

use super::{identifier::identifier, spaced, span, type_signature::type_signature, ws, Input, Res};

pub fn external_object(i: Input<'_>) -> Res<Input<'_>, ExternalObject<'_>> {
    // external NAME: TYPE_SIG

    map(
        span(pair(
            preceded(tuple((multispace0, tag("external"), ws)), identifier),
            preceded(spaced(tag(":")), type_signature),
        )),
        |(span, (ident, type_sig))| ExternalObject {
            ident,
            type_sig,
            span,
        },
    )(i)
}
