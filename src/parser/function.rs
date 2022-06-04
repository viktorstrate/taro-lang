use nom::{
    bytes::complete::tag,
    combinator::opt,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::ast::node::{
    function::{Function, FunctionArg},
    type_signature::BuiltinType,
};

use super::{
    statement::{identifier, statement, type_signature},
    surround_brackets, token, ws, BracketType, Res, Span,
};

pub fn function(i: Span) -> Res<Span, Function> {
    // func IDENT "(" FUNC_ARGS ")" [-> RETURN_SIG] "{" BODY "}"

    let (i, _) = token(tuple((tag("func"), ws)))(i)?;
    let (i, name) = identifier(i)?;

    let (i, args) = surround_brackets(BracketType::Round, function_args)(i)?;

    let (i, return_type) = opt(preceded(token(tag("->")), type_signature))(i)?;

    let (i, body) = surround_brackets(BracketType::Curly, statement)(i)?;

    Ok((
        i,
        Function {
            name,
            args,
            return_type: return_type.unwrap_or(BuiltinType::Void.into()),
            body,
        },
    ))
}

fn function_args(i: Span) -> Res<Span, Vec<FunctionArg>> {
    separated_list0(token(tag(",")), function_arg)(i)
}

fn function_arg(i: Span) -> Res<Span, FunctionArg> {
    // IDENT : TYPE_SIG

    let (i, name) = identifier(i)?;
    let (i, _) = token(tag(":"))(i)?;
    let (i, type_sig) = type_signature(i)?;

    Ok((i, FunctionArg { name, type_sig }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        let func = function(Span::new("func sum (a: Number, b: Number) -> Number {}"))
            .unwrap()
            .1;

        assert_eq!(func.name.value, "sum");
        assert_eq!(func.return_type, BuiltinType::Number.into());
        assert_eq!(func.args.len(), 2);
        assert_eq!(func.args[0].name.value, "a");
        assert_eq!(func.args[1].name.value, "b");
        assert_eq!(func.args[0].type_sig, BuiltinType::Number.into());
        assert_eq!(func.args[1].type_sig, BuiltinType::Number.into());
    }
}
