use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    error::VerboseError,
    sequence::{delimited, preceded},
    AsChar, Finish, IResult, InputTakeAtPosition,
};
use nom_locate::{position, LocatedSpan};

use crate::ast::AST;

pub mod enumeration;
pub mod escape_block;
pub mod expression;
pub mod function;
pub mod identifier;
pub mod module;
pub mod statement;
pub mod structure;
pub mod type_signature;

pub fn parse_ast(input: &str) -> Result<AST, ParserError> {
    match module::module(new_input(input)).finish() {
        Ok((_, module)) => Ok(AST::from(module)),
        Err(err) => Err(err),
    }
}

pub type ParserError<'a> = VerboseError<Input<'a>>;

pub type Res<I, O> = IResult<I, O, VerboseError<I>>;

#[derive(Debug, Default, Clone)]
pub struct ParserContext();

pub type Input<'a> = LocatedSpan<&'a str, ParserContext>;

pub fn ws(i: Input) -> Res<Input, Input> {
    return multispace1(i);
}

pub fn new_input(input: &str) -> Input {
    Input::new_extra(input, ParserContext::default())
}

pub fn token<F, I, O>(mut parser: F) -> impl FnMut(I) -> Res<I, O>
where
    F: FnMut(I) -> Res<I, O>,
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    return move |i: I| {
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
        token(tag(brackets.open())),
        parser,
        preceded(multispace0, tag(brackets.close())),
    )
}

#[derive(Debug, Clone)]
pub struct Span<'a> {
    pub line: usize,
    pub offset: usize,
    pub fragment: &'a str,
}

impl<'a> Span<'a> {
    pub fn new(start: Input<'a>, end: Input<'a>) -> Span<'a> {
        let len = end.location_offset() - start.location_offset();

        Span {
            line: start.location_line() as usize,
            offset: start.get_utf8_column(),
            fragment: &start.fragment()[0..len],
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
