use std::collections::HashMap;

use crate::ir::{
    ir_walker::IrWalker,
    node::type_signature::{TypeSignature, Typed},
};

#[derive(Debug)]
pub struct TypeConstraint<'a>(Box<dyn Typed<'a>>, Box<dyn Typed<'a>>);

#[derive(Debug, Default)]
pub struct TypeInferrer<'a> {
    pub substitutions: HashMap<TypeSignature<'a>, TypeSignature<'a>>,
    pub constraints: Vec<TypeConstraint<'a>>,
}

#[derive(Debug)]
pub enum TypeInferenceError<'a> {
    ConflictingTypes(TypeSignature<'a>, TypeSignature<'a>),
}

impl<'a> IrWalker<'a> for TypeInferrer<'a> {
    type Error = TypeInferenceError<'a>;
    type Scope = ();
}
