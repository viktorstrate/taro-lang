use std::cmp::Ordering;

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0, multispace1},
    error::VerboseError,
    sequence::{delimited, preceded},
    IResult,
};
use nom_locate::{position, LocatedSpan};

use crate::ast::AST;

pub mod comment;
pub mod control_flow;
pub mod enumeration;
pub mod escape_block;
pub mod expression;
pub mod external;
pub mod function;
pub mod identifier;
pub mod module;
pub mod statement;
pub mod structure;
pub mod type_signature;

pub fn parse_ast(input: &str) -> Result<AST<'_>, ParserError<'_>> {
    match module::module(new_input(input)) {
        Ok(module) => Ok(AST::from(module)),
        Err(err) => Err(err),
    }
}

pub type ParserError<'a> = VerboseError<Input<'a>>;

pub type Res<I, O> = IResult<I, O, VerboseError<I>>;

#[derive(Debug, Default, Clone)]
pub struct ParserContext<'a> {
    source: &'a str,
}

pub type Input<'a> = LocatedSpan<&'a str, ParserContext<'a>>;

pub fn ws(i: Input<'_>) -> Res<Input<'_>, Input<'_>> {
    return multispace1(i);
}

pub fn new_input(input: &str) -> Input<'_> {
    Input::new_extra(input, ParserContext { source: input })
}

pub fn spaced<'a, F, O>(mut parser: F) -> impl FnMut(Input<'a>) -> Res<Input<'a>, O>
where
    F: FnMut(Input<'a>) -> Res<Input<'a>, O>,
    // I: InputTakeAtPosition,
    // <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    return move |i: Input<'a>| {
        let (i, _) = multispace0(i)?;
        let (i, res) = parser(i)?;
        let (i, _) = multispace0(i)?;

        return Ok((i, res));
    };
}

pub enum BracketType {
    Round,
    Square,
    Curly,
}

impl BracketType {
    pub fn open(&self) -> &'static str {
        match self {
            BracketType::Round => "(",
            BracketType::Square => "[",
            BracketType::Curly => "{",
        }
    }

    pub fn close(&self) -> &'static str {
        match self {
            BracketType::Round => ")",
            BracketType::Square => "]",
            BracketType::Curly => "}",
        }
    }
}

pub fn surround_brackets<'a, F, O>(
    brackets: BracketType,
    parser: F,
) -> impl FnMut(Input<'a>) -> Res<Input<'a>, O>
where
    F: FnMut(Input<'a>) -> Res<Input<'a>, O>,
{
    delimited(
        spaced(tag(brackets.open())),
        parser,
        preceded(multispace0, tag(brackets.close())),
    )
}

pub fn decimal(i: Input<'_>) -> Res<Input<'_>, Input<'_>> {
    digit1(i)
}

#[derive(Debug, Clone, Eq)]
pub struct Span<'a> {
    pub line: usize,
    pub offset: usize,
    pub fragment: &'a str,
    pub source: &'a str,
}

impl<'a> Ord for Span<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a> PartialOrd for Span<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.line.partial_cmp(&other.line) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.offset.partial_cmp(&other.offset) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        Some(Ordering::Equal)
    }
}

impl<'a> Span<'a> {
    pub fn new(start: Input<'a>, end: Input<'a>) -> Span<'a> {
        let len = end.location_offset() - start.location_offset();

        // trim span
        let mut fragment = &start.fragment()[0..len];
        let trim_start = fragment.chars().take_while(|c| c.is_whitespace()).count();
        let trim_end = fragment
            .chars()
            .rev()
            .take_while(|c| c.is_whitespace())
            .count();

        fragment = &fragment[trim_start..(len - trim_end)];

        Span {
            line: start.location_line() as usize,
            offset: start.get_utf8_column() + trim_start,
            fragment,
            source: start.extra.source,
        }
    }

    pub fn empty() -> Span<'a> {
        Span {
            line: 0,
            offset: 0,
            fragment: "",
            source: "",
        }
    }
}

// Equality comparisons should not consider span attributes
impl PartialEq for Span<'_> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

pub fn span<'a, F, O>(mut parser: F) -> impl FnMut(Input<'a>) -> Res<Input<'a>, (Span<'a>, O)>
where
    F: FnMut(Input<'a>) -> Res<Input<'a>, O>,
{
    return move |i: Input<'a>| {
        let (start_i, _) = position(i)?;
        let (parsed_i, out) = parser(start_i.clone())?;
        let (i, end) = position(parsed_i.clone())?;

        Ok((i, (Span::new(start_i, end), out)))
    };
}
