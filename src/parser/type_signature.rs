use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    error::context,
    multi::separated_list0,
    sequence::{preceded, tuple},
};

use crate::ast::node::type_signature::{TypeSignature, TypeSignatureValue};

use super::{identifier::identifier, spaced, span, surround_brackets, BracketType, Input, Res};

pub fn type_signature(i: Input<'_>) -> Res<Input<'_>, TypeSignature<'_>> {
    context(
        "type signature",
        map(
            span(alt((type_sig_func, type_sig_base, type_sig_tuple))),
            |(span, value)| TypeSignature { span, value },
        ),
    )(i)
}

fn type_sig_base(i: Input<'_>) -> Res<Input<'_>, TypeSignatureValue<'_>> {
    // IDENT

    context("base type", map(identifier, TypeSignatureValue::Base))(i)
}

fn type_sig_func(i: Input<'_>) -> Res<Input<'_>, TypeSignatureValue<'_>> {
    // "(" TYPE_SIG , ... ")" "->" TYPE_SIG

    context(
        "function type",
        map(
            tuple((
                surround_brackets(
                    BracketType::Round,
                    separated_list0(spaced(tag(",")), type_signature),
                ),
                preceded(spaced(tag("->")), type_signature),
            )),
            |(args, return_type)| TypeSignatureValue::Function {
                args,
                return_type: Box::new(return_type),
            },
        ),
    )(i)
}

fn type_sig_tuple(i: Input<'_>) -> Res<Input<'_>, TypeSignatureValue<'_>> {
    // "(" TYPE_SIG , ... ")"

    context(
        "tuple type",
        map(
            surround_brackets(
                BracketType::Round,
                separated_list0(spaced(tag(",")), type_signature),
            ),
            TypeSignatureValue::Tuple,
        ),
    )(i)
}

#[cfg(test)]
mod tests {

    use std::assert_matches::assert_matches;

    use crate::{
        ast::{node::identifier::Ident, test_utils::test_type_sig},
        parser::{new_input, Span},
    };

    use super::*;

    #[test]
    fn test_base_type() {
        assert_matches!(
            type_signature(new_input("Boolean")).unwrap().1,
            TypeSignature {
                span: Span {
                    line: _,
                    offset: _,
                    fragment: "Boolean",
                    source: _
                },
                value: TypeSignatureValue::Base(Ident {
                    span: _,
                    value: "Boolean"
                })
            }
        );
    }

    #[test]
    fn test_function_type() {
        let func_type = type_signature(new_input("() -> Void")).unwrap().1;

        match func_type {
            TypeSignature {
                span,
                value: TypeSignatureValue::Function { args, return_type },
            } => {
                assert_eq!(span.fragment, "() -> Void");
                assert!(args.is_empty());
                assert_eq!(*return_type, test_type_sig("Void"));
                assert_eq!(return_type.span.fragment, "Void");
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_nested_function_type() {
        let func_type = type_signature(new_input("(Number, (Number) -> Boolean) -> Boolean"))
            .unwrap()
            .1;

        match func_type {
            TypeSignature {
                span,
                value: TypeSignatureValue::Function { args, return_type },
            } => {
                assert_eq!(span.fragment, "(Number, (Number) -> Boolean) -> Boolean");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], test_type_sig("Number"));
                assert_eq!(*return_type, test_type_sig("Boolean"));
                match &args[1] {
                    TypeSignature {
                        span,
                        value: TypeSignatureValue::Function { args, return_type },
                    } => {
                        assert_eq!(span.fragment, "(Number) -> Boolean");
                        assert_eq!(args.len(), 1);
                        assert_eq!(args[0], test_type_sig("Number"));
                        assert_eq!(*return_type.as_ref(), test_type_sig("Boolean"));
                    }
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_tuple_type() {
        assert_eq!(
            type_signature(new_input("()")).unwrap().1.value,
            TypeSignatureValue::Tuple(Vec::new())
        );

        let pair = type_signature(new_input("(Number, String)")).unwrap().1;

        match pair {
            TypeSignature {
                span,
                value: TypeSignatureValue::Tuple(types),
            } => {
                assert_eq!(span.fragment, "(Number, String)");
                assert_eq!(types.len(), 2);
                assert_eq!(types[0].span.fragment, "Number");
                assert_eq!(types[1].span.fragment, "String");
            }
            _ => assert!(false),
        }
    }
}
