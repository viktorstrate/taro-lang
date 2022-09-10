use std::collections::{HashMap, VecDeque};

use crate::{
    ir::{
        context::IrCtx,
        ir_walker::walk_ir,
        node::{
            enumeration::{EnumInit, EnumValue},
            function::FunctionCall,
            identifier::Ident,
            type_signature::{TypeEvalError, TypeSignature},
            NodeRef,
        },
        IR,
    },
    symbols::{
        symbol_resolver::SymbolResolver, symbol_table::symbol_table_zipper::SymbolTableZipper,
    },
};

use self::{
    check_assignment::AssignmentError,
    check_struct::StructTypeError,
    type_inference::{TypeConstraint, TypeInferrer},
    type_resolver::TypeResolver,
    types_walker::EndTypeChecker,
};

pub mod check_assignment;
pub mod check_enum;
pub mod check_struct;
pub mod coercion;
pub mod type_inference;
pub mod type_resolver;
pub mod types_walker;

#[derive(Debug)]
pub enum TypeCheckerError<'a> {
    ConflictingTypes(TypeSignature<'a>, TypeSignature<'a>),
    UndeterminableTypes,
    TypeEval(TypeEvalError<'a>),
    LookupError(Ident<'a>),
    AssignmentError(AssignmentError<'a>),
    StructError(StructTypeError<'a>),
    FuncArgCountMismatch(TypeSignature<'a>, TypeSignature<'a>),
    FuncCallWrongArgAmount(NodeRef<'a, FunctionCall<'a>>),
    UnknownEnumValue {
        enum_name: Ident<'a>,
        enum_value: Ident<'a>,
    },
    EnumInitArgCountMismatch(NodeRef<'a, EnumInit<'a>>, NodeRef<'a, EnumValue<'a>>),
}

#[derive(Debug)]
pub struct TypeChecker<'a> {
    pub symbols: SymbolTableZipper<'a>,
    pub substitutions: HashMap<TypeSignature<'a>, TypeSignature<'a>>,
    pub constraints: VecDeque<TypeConstraint<'a>>,
    pub needs_rerun: bool,
    pub found_undeterminable_types: bool,
}

impl<'a> TypeChecker<'a> {
    pub fn new(ctx: &IrCtx<'a>, sym_resolver: SymbolResolver<'a>) -> Self {
        let mut symbols = sym_resolver.symbols;
        symbols.reset(ctx);
        TypeChecker {
            symbols,
            substitutions: HashMap::new(),
            constraints: VecDeque::new(),
            needs_rerun: true,
            found_undeterminable_types: false,
        }
    }

    pub fn type_check(
        &mut self,
        ctx: &mut IrCtx<'a>,
        ir: &mut IR<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        while self.needs_rerun {
            self.needs_rerun = false;
            self.found_undeterminable_types = false;
            self.substitutions.clear();
            self.constraints.clear();

            let mut type_inferrer = TypeInferrer::new(ctx, self);
            walk_ir(&mut type_inferrer, ctx, ir)?;

            let mut type_resolver = TypeResolver::new(&ctx, &mut type_inferrer);
            walk_ir(&mut type_resolver, ctx, ir)?;

            let mut type_checker = EndTypeChecker::new(&ctx, &mut type_resolver);
            walk_ir(&mut type_checker, ctx, ir)?;
        }

        if self.found_undeterminable_types {
            return Err(TypeCheckerError::UndeterminableTypes);
        }

        Ok(())
    }
}
