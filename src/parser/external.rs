use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{pair, preceded},
};

use crate::ast::node::external::ExternalObject;

use super::{identifier::identifier, spaced, span, type_signature::type_signature, ws, Input, Res};

pub fn external_object(i: Input<'_>) -> Res<Input<'_>, ExternalObject<'_>> {
    // external NAME: TYPE_SIG

    map(
        span(pair(
            preceded(pair(tag("external"), ws), identifier),
            preceded(spaced(tag(":")), type_signature),
        )),
        |(span, (ident, type_sig))| ExternalObject {
            ident,
            type_sig,
            span,
        },
    )(i)
}
