use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    error::context,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::ast::node::type_signature::TypeSignature;

use super::{identifier::identifier, surround_brackets, token, BracketType, Res, Span};

pub fn type_signature(i: Span) -> Res<Span, TypeSignature> {
    context(
        "type signature",
        alt((type_sig_func, type_sig_base, type_sig_tuple)),
    )(i)
}

fn type_sig_base(i: Span) -> Res<Span, TypeSignature> {
    // IDENT

    context("base type", map(identifier, TypeSignature::Base))(i)
}

fn type_sig_func(i: Span) -> Res<Span, TypeSignature> {
    // "(" TYPE_SIG , ... ")" "->" TYPE_SIG

    context(
        "function type",
        map(
            tuple((
                surround_brackets(
                    BracketType::Round,
                    separated_list0(token(tag(",")), type_signature),
                ),
                preceded(token(tag("->")), type_signature),
            )),
            |(args, return_type)| TypeSignature::Function {
                args,
                return_type: Box::new(return_type),
            },
        ),
    )(i)
}

fn type_sig_tuple(i: Span) -> Res<Span, TypeSignature> {
    // "(" TYPE_SIG , ... ")"

    context(
        "tuple type",
        map(
            surround_brackets(
                BracketType::Round,
                separated_list0(token(tag(",")), type_signature),
            ),
            TypeSignature::Tuple,
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use crate::{parser::new_span, symbols::builtin_types::BuiltinType};

    use super::*;

    #[test]
    fn test_base_type() {
        assert_eq!(
            type_signature(new_span("Boolean")).unwrap().1,
            BuiltinType::Boolean.type_sig()
        );
    }

    #[test]
    fn test_function_type() {
        assert_eq!(
            type_signature(new_span("() -> Void")).unwrap().1,
            TypeSignature::Function {
                args: vec![],
                return_type: Box::new(BuiltinType::Void.type_sig())
            }
        );
    }

    #[test]
    fn test_nested_function_type() {
        assert_eq!(
            type_signature(new_span("(Number, (Number) -> Boolean) -> Boolean"))
                .unwrap()
                .1,
            TypeSignature::Function {
                args: vec![
                    BuiltinType::Number.type_sig(),
                    TypeSignature::Function {
                        args: vec![BuiltinType::Number.type_sig()],
                        return_type: Box::new(BuiltinType::Boolean.type_sig())
                    }
                ],
                return_type: Box::new(BuiltinType::Boolean.type_sig())
            }
        );
    }

    #[test]
    fn test_tuple_type() {
        assert_eq!(
            type_signature(new_span("()")).unwrap().1,
            TypeSignature::Tuple(Vec::new())
        );

        assert_eq!(
            type_signature(new_span("(Number, String)")).unwrap().1,
            TypeSignature::Tuple(vec![
                BuiltinType::Number.type_sig(),
                BuiltinType::String.type_sig()
            ])
        );
    }
}
