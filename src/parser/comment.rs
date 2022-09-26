use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, not_line_ending},
    combinator::map,
    sequence::{pair, preceded, tuple},
};

use crate::ast::node::comment::Comment;

use super::{spaced, Input, Res};

pub fn comment(i: Input<'_>) -> Res<Input<'_>, Comment<'_>> {
    alt((line_comment, block_comment))(i)
}

pub fn line_comment<'a>(i: Input<'a>) -> Res<Input<'a>, Comment<'a>> {
    map(
        preceded(pair(multispace0, tag("//")), not_line_ending),
        |val: Input<'a>| Comment::Line(val.as_ref()),
    )(i)
}

pub fn block_comment(i: Input<'_>) -> Res<Input<'_>, Comment<'_>> {
    map(
        spaced(tuple((
            tag("/*"),
            not_line_ending::<Input<'_>, _>,
            tag("*/"),
        ))),
        |(_, content, _)| Comment::Line(content.as_ref()),
    )(i)
}
