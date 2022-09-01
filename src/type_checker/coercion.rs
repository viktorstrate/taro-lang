use crate::ir::{
    context::IrCtx,
    node::type_signature::{TypeSignature, TypeSignatureValue},
};

pub fn can_coerce_to<'a>(
    type_sig: TypeSignature<'a>,
    other: TypeSignature<'a>,
    ctx: &IrCtx<'a>,
) -> bool {
    let self_t = &ctx[type_sig];
    let other_t = &ctx[other];

    if let (TypeSignatureValue::Tuple(selves), TypeSignatureValue::Tuple(others)) =
        (self_t, other_t)
    {
        selves
            .iter()
            .zip(others.iter())
            .all(|(slf, other)| can_coerce_to(*slf, *other, ctx))
    } else {
        type_sig == other
    }
}

pub fn coerce<'a>(
    a: TypeSignature<'a>,
    b: TypeSignature<'a>,
    ctx: &IrCtx<'a>,
) -> Option<TypeSignature<'a>> {
    if can_coerce_to(a, b, ctx) {
        Some(b)
    } else if can_coerce_to(b, a, ctx) {
        Some(a)
    } else {
        None
    }
}
