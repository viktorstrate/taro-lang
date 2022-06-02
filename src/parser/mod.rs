use nom::{
    character::complete::{multispace0, multispace1},
    combinator::complete,
    error::VerboseError,
    AsChar, IResult, InputTakeAtPosition,
};
use nom_locate::LocatedSpan;

use crate::ast::AST;

use self::statements::statement;

pub mod expressions;
pub mod statements;

pub fn parse_ast(input: &str) -> Result<AST, nom::Err<VerboseError<Span>>> {
    match complete(statement)(Span::new(input)) {
        Ok((_, stmt)) => Ok(AST::from(stmt)),
        Err(err) => Err(err),
    }
}

pub type Res<I, O> = IResult<I, O, VerboseError<I>>;

pub type Span<'a> = LocatedSpan<&'a str>;

pub fn ws(i: &str) -> Res<&str, &str> {
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
