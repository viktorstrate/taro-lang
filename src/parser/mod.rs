use nom::{
    character::complete::{multispace0, multispace1},
    error::VerboseError,
    AsChar, IResult, InputTakeAtPosition,
};

pub mod expressions;
pub mod statements;

pub type Res<I, O> = IResult<I, O, VerboseError<I>>;

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
