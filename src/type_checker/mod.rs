use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use crate::{
    ir::{
        ast_lowering::LowerAstResult,
        context::IrCtx,
        ir_walker::walk_ir,
        node::{
            assignment::Assignment,
            enumeration::{EnumInit, EnumValue},
            expression::Expr,
            function::FunctionCall,
            member_access::UnresolvedMemberAccess,
            structure::Struct,
            type_signature::{TypeEvalError, TypeSignature},
            NodeRef,
        },
    },
    parser::Span,
    symbols::{
        symbol_resolver::{SymbolResolutionError, SymbolResolver},
        symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolValue},
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
pub mod check_expr_ident;
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
    AssignmentError(NodeRef<'a, Assignment<'a>>, AssignmentError<'a>),
    StructError(NodeRef<'a, Struct<'a>>, StructTypeError<'a>),
    FunctionError(FunctionError<'a>),
    EnumInitArgCountMismatch(NodeRef<'a, EnumInit<'a>>, NodeRef<'a, EnumValue<'a>>),
    AnonymousEnumInitNonEnum(NodeRef<'a, UnresolvedMemberAccess<'a>>, TypeSignature<'a>),
    UnresolvableTypeConstraints(VecDeque<TypeConstraint<'a>>),
    UndeterminableTypes(Vec<UndeterminableType<'a>>),
    IdentNotExpression(NodeRef<'a, Expr<'a>>, SymbolValue<'a>),
}

#[derive(Debug)]
pub enum FunctionError<'a> {
    ArgCountMismatch(TypeSignature<'a>, TypeSignature<'a>),
    FuncCallWrongArgAmount {
        call: NodeRef<'a, FunctionCall<'a>>,
        func_type: TypeSignature<'a>,
    },
}

#[derive(Debug, Clone)]
pub enum ExpectedType {
    Enum,
    Struct,
}

impl Display for ExpectedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedType::Enum => write!(f, "enum"),
            ExpectedType::Struct => write!(f, "struct"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UndeterminableType<'a> {
    pub span: Span<'a>,
    pub expected: ExpectedType,
}

#[derive(Debug)]
pub struct TypeChecker<'a> {
    pub symbols: SymbolTableZipper<'a>,
    pub substitutions: HashMap<TypeSignature<'a>, TypeSignature<'a>>,
    pub constraints: VecDeque<TypeConstraint<'a>>,
    pub previous_undeterminable_types: Vec<UndeterminableType<'a>>,
    pub immediate_undeterminable_types: Vec<UndeterminableType<'a>>,
    pub needs_rerun: bool,
}

impl<'a> TypeChecker<'a> {
    pub fn new(ctx: &IrCtx<'a>, sym_resolver: SymbolResolver<'a>) -> Self {
        let mut symbols = sym_resolver.symbols;
        symbols.reset(ctx);
        TypeChecker {
            symbols,
            substitutions: HashMap::new(),
            constraints: VecDeque::new(),
            previous_undeterminable_types: Vec::new(),
            immediate_undeterminable_types: Vec::new(),
            needs_rerun: true,
        }
    }

    pub fn type_check(&mut self, la: &mut LowerAstResult<'a>) -> Result<(), TypeCheckerError<'a>> {
        while self.needs_rerun {
            self.needs_rerun = false;
            self.substitutions.clear();
            self.constraints.clear();

            self.previous_undeterminable_types.clear();
            std::mem::swap(
                &mut self.previous_undeterminable_types,
                &mut self.immediate_undeterminable_types,
            );

            let mut type_inferrer = TypeInferrer::new(&la.ctx, self);
            walk_ir(&mut type_inferrer, la)?;

            let mut type_resolver = TypeResolver::new(&la.ctx, &mut type_inferrer);
            walk_ir(&mut type_resolver, la)?;

            if !type_resolver.0.needs_rerun {
                let mut type_checker = EndTypeChecker::new(&la.ctx, &mut type_resolver);
                walk_ir(&mut type_checker, la)?;
            }
        }

        if !self.constraints.is_empty() {
            let mut x = VecDeque::new();
            std::mem::swap(&mut x, &mut self.constraints);
            return Err(TypeCheckerError::UnresolvableTypeConstraints(x));
        }

        if !self.immediate_undeterminable_types.is_empty() {
            let mut x = Vec::new();
            std::mem::swap(&mut x, &mut self.immediate_undeterminable_types);
            return Err(TypeCheckerError::UndeterminableTypes(x));
        }

        Ok(())
    }

    #[inline]
    fn add_constraint(&mut self, a: TypeSignature<'a>, b: TypeSignature<'a>) {
        self.constraints.push_back(TypeConstraint(a, b))
    }
}
