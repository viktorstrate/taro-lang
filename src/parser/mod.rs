use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    error::VerboseError,
    sequence::{delimited, preceded},
    AsChar, Finish, IResult, InputTakeAtPosition,
};
use nom_locate::LocatedSpan;

use crate::ast::{ref_generator::RefGen, AST};

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
    match module::module(new_span(input)).finish() {
        Ok((_, module)) => Ok(AST::from(module)),
        Err(err) => Err(err),
    }
}

pub type ParserError<'a> = VerboseError<Span<'a>>;

pub type Res<I, O> = IResult<I, O, VerboseError<I>>;

#[derive(Debug, Default, Clone)]
pub struct ParserContext {
    pub ref_gen: RefGen,
}

pub type Span<'a> = LocatedSpan<&'a str, ParserContext>;

pub fn ws(i: Span) -> Res<Span, Span> {
    return multispace1(i);
}

pub fn new_span(input: &str) -> Span {
    Span::new_extra(input, ParserContext::default())
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
        preceded(multispace0, tag(brackets.close())),
    )
}
