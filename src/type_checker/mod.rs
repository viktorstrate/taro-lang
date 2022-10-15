use std::collections::{HashMap, VecDeque};

use crate::{
    ir::{
        ast_lowering::LowerAstResult,
        context::IrCtx,
        ir_walker::walk_ir,
        node::{
            assignment::Assignment,
            enumeration::{EnumInit, EnumValue},
            function::FunctionCall,
            member_access::UnresolvedMemberAccess,
            structure::Struct,
            type_signature::{TypeEvalError, TypeSignature},
            NodeRef,
        },
    },
    symbols::{
        symbol_resolver::{SymbolResolutionError, SymbolResolver},
        symbol_table::symbol_table_zipper::SymbolTableZipper,
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
    SymbolResolutionError(SymbolResolutionError<'a>),
    TypeEval(TypeEvalError<'a>),
    ConflictingTypes(TypeSignature<'a>, TypeSignature<'a>),
    UndeterminableTypes,
    AssignmentError(NodeRef<'a, Assignment<'a>>, AssignmentError<'a>),
    StructError(NodeRef<'a, Struct<'a>>, StructTypeError<'a>),
    FunctionError(FunctionError<'a>),
    EnumInitArgCountMismatch(NodeRef<'a, EnumInit<'a>>, NodeRef<'a, EnumValue<'a>>),
    AnonymousEnumInitNonEnum(NodeRef<'a, UnresolvedMemberAccess<'a>>, TypeSignature<'a>),
}

#[derive(Debug)]
pub enum FunctionError<'a> {
    ArgCountMismatch(TypeSignature<'a>, TypeSignature<'a>),
    FuncCallWrongArgAmount {
        call: NodeRef<'a, FunctionCall<'a>>,
        func_type: TypeSignature<'a>,
    },
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

    pub fn type_check(&mut self, la: &mut LowerAstResult<'a>) -> Result<(), TypeCheckerError<'a>> {
        while self.needs_rerun {
            self.needs_rerun = false;
            self.found_undeterminable_types = false;
            self.substitutions.clear();
            self.constraints.clear();

            let mut type_inferrer = TypeInferrer::new(&la.ctx, self);
            walk_ir(&mut type_inferrer, la)?;

            let mut type_resolver = TypeResolver::new(&la.ctx, &mut type_inferrer);
            walk_ir(&mut type_resolver, la)?;

            if !type_resolver.0.needs_rerun {
                let mut type_checker = EndTypeChecker::new(&la.ctx, &mut type_resolver);
                walk_ir(&mut type_checker, la)?;
            }
        }

        if self.found_undeterminable_types {
            return Err(TypeCheckerError::UndeterminableTypes);
        }

        Ok(())
    }
}
