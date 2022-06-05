use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::cut,
    error::VerboseError,
    sequence::delimited,
    AsChar, Finish, IResult, InputTakeAtPosition,
};
use nom_locate::LocatedSpan;

use crate::ast::AST;

pub mod expression;
pub mod function;
pub mod identifier;
pub mod module;
pub mod statement;
pub mod structure;

pub fn parse_ast(input: &str) -> Result<AST, ParserError> {
    match module::module(Span::new(input)).finish() {
        Ok((_, module)) => Ok(AST::from(module)),
        Err(err) => Err(err),
    }
}

pub type ParserError<'a> = VerboseError<Span<'a>>;

pub type Res<I, O> = IResult<I, O, VerboseError<I>>;

pub type Span<'a> = LocatedSpan<&'a str>;

pub fn ws(i: Span) -> Res<Span, Span> {
    return multispace1(i);
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
) -> impl FnMut(Span<'a>) -> Res<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> Res<Span<'a>, O>,
{
    delimited(
        token(tag(brackets.open())),
        parser,
        cut(token(tag(brackets.close()))),
    )
}
