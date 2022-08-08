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
use std::{convert::Into, marker::PhantomData};

use super::context::{IrArenaType, IrCtx};

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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NodeRef<'a, T>
where
    T: IrArenaType<'a>,
{
    id: Id<T>,
    _marker: PhantomData<&'a str>,
}

impl<'a, T> Copy for NodeRef<'a, T> where T: IrArenaType<'a> {}

impl<'a, T> Clone for NodeRef<'a, T>
where
    T: IrArenaType<'a>,
{
    fn clone(&self) -> Self {
        NodeRef {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> From<Id<T>> for NodeRef<'a, T>
where
    T: IrArenaType<'a>,
{
    fn from(val: Id<T>) -> Self {
        NodeRef {
            id: val,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Into<Id<T>> for NodeRef<'a, T>
where
    T: IrArenaType<'a>,
{
    fn into(self) -> Id<T> {
        self.id
    }
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
    Self: IrArenaType<'a>,
{
    fn allocate(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self>;
}

impl<'a, T> IrAlloc<'a> for T
where
    T: IrArenaType<'a>,
{
    fn allocate(self, ctx: &mut IrCtx<'a>) -> NodeRef<'a, Self> {
        let id = Self::arena_mut(ctx).alloc(self);
        NodeRef::from(id)
    }
}

impl<'a> IrArenaType<'a> for Stmt<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.stmts
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.stmts
    }
}

impl<'a> IrArenaType<'a> for Expr<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.exprs
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.exprs
    }
}

impl<'a> IrArenaType<'a> for FunctionArg<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.func_args
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.func_args
    }
}

impl<'a> IrArenaType<'a> for StructAttr<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_attrs
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_attrs
    }
}

impl<'a> IrArenaType<'a> for Enum<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.enms
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.enms
    }
}

impl<'a> IrArenaType<'a> for EnumValue<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.enm_vals
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.enm_vals
    }
}

impl<'a> IrArenaType<'a> for Function<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.funcs
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.funcs
    }
}

impl<'a> IrArenaType<'a> for FunctionCall<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.func_calls
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.func_calls
    }
}

impl<'a> IrArenaType<'a> for Struct<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_decls
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_decls
    }
}

impl<'a> IrArenaType<'a> for StructInitValue<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_init_vals
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_init_vals
    }
}

impl<'a> IrArenaType<'a> for StructInit<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_inits
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_inits
    }
}

impl<'a> IrArenaType<'a> for StructAccess<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_accs
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_accs
    }
}

impl<'a> IrArenaType<'a> for Tuple<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.tups
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.tups
    }
}

impl<'a> IrArenaType<'a> for TupleAccess<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.tup_accs
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.tup_accs
    }
}

impl<'a> IrArenaType<'a> for Assignment<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.asgns
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.asgns
    }
}

impl<'a> IrArenaType<'a> for EscapeBlock<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.esc_blks
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.esc_blks
    }
}

impl<'a> IrArenaType<'a> for VarDecl<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.var_decls
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.var_decls
    }
}
