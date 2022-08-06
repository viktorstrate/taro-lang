use id_arena::{Arena, Id};

use self::{
    assignment::Assignment,
    enumeration::{Enum, EnumValue},
    escape_block::EscapeBlock,
    expression::Expr,
    function::{Function, FunctionArg, FunctionCall},
    statement::{Stmt, VarDecl},
    structure::{Struct, StructAccess, StructAttr, StructInit, StructInitValue},
    tuple::{Tuple, TupleAccess},
};

use super::context::IrCtx;

pub mod assignment;
pub mod enumeration;
pub mod escape_block;
pub mod expression;
pub mod function;
pub mod identifier;
pub mod module;
pub mod statement;
pub mod structure;
pub mod tuple;
pub mod type_signature;

pub enum IrNode<'a> {
    Stmt(Stmt<'a>),
    Expr(Expr<'a>),
    FunctionArg(FunctionArg<'a>),
    StructAttr(StructAttr<'a>),
    EnumValue(EnumValue<'a>),
    Function(Function<'a>),
    FunctionCall(FunctionCall<'a>),
    StructInitValue(StructInitValue<'a>),
    StructInit(StructInit<'a>),
    StructAccess(StructAccess<'a>),
    TupleAccess(TupleAccess<'a>),
    Tuple(Tuple<'a>),
    Assignment(Assignment<'a>),
    EscapeBlock(EscapeBlock<'a>),
}

pub struct IrNodeArena<'a> {
    pub stmts: Arena<Stmt<'a>>,
    pub exprs: Arena<Expr<'a>>,
    pub func_args: Arena<FunctionArg<'a>>,
    pub st_attrs: Arena<StructAttr<'a>>,
    pub enms: Arena<Enum<'a>>,
    pub enm_vals: Arena<EnumValue<'a>>,
    pub funcs: Arena<Function<'a>>,
    pub func_calls: Arena<FunctionCall<'a>>,
    pub st_decls: Arena<Struct<'a>>,
    pub st_init_vals: Arena<StructInitValue<'a>>,
    pub st_inits: Arena<StructInit<'a>>,
    pub st_accs: Arena<StructAccess<'a>>,
    pub tup_accs: Arena<TupleAccess<'a>>,
    pub tups: Arena<Tuple<'a>>,
    pub asgns: Arena<Assignment<'a>>,
    pub esc_blks: Arena<EscapeBlock<'a>>,
    pub var_decls: Arena<VarDecl<'a>>,
}

impl<'a> IrNodeArena<'a> {
    pub fn new() -> Self {
        todo!()
    }
}

pub trait IrAlloc<'a>
where
    Self: Sized,
{
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self>;
}

impl<'a> IrAlloc<'a> for Stmt<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.stmts.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for Expr<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.exprs.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for FunctionArg<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.func_args.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for StructAttr<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.st_attrs.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for Enum<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.enms.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for EnumValue<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.enm_vals.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for Function<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.funcs.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for FunctionCall<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.func_calls.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for Struct<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.st_decls.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for StructInitValue<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.st_init_vals.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for StructInit<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.st_inits.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for StructAccess<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.st_accs.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for Tuple<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.tups.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for TupleAccess<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.tup_accs.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for Assignment<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.asgns.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for EscapeBlock<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.esc_blks.alloc(self)
    }
}

impl<'a> IrAlloc<'a> for VarDecl<'a> {
    fn allocate(self, ctx: &mut IrCtx<'a>) -> Id<Self> {
        ctx.nodes.var_decls.alloc(self)
    }
}
