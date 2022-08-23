use id_arena::{Arena, Id};

use self::{
    assignment::Assignment,
    enumeration::{Enum, EnumInit, EnumValue},
    escape_block::EscapeBlock,
    expression::Expr,
    function::{Function, FunctionArg, FunctionCall},
    member_access::UnresolvedMemberAccess,
    statement::{Stmt, StmtBlock, VarDecl},
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
pub mod member_access;
pub mod module;
pub mod statement;
pub mod structure;
pub mod tuple;
pub mod type_signature;

// pub enum IrNode<'a> {
//     Stmt(Stmt<'a>),
//     Expr(Expr<'a>),
//     FunctionArg(FunctionArg<'a>),
//     StructAttr(StructAttr<'a>),
//     EnumValue(EnumValue<'a>),
//     Function(Function<'a>),
//     FunctionCall(FunctionCall<'a>),
//     StructInitValue(StructInitValue<'a>),
//     StructInit(StructInit<'a>),
//     StructAccess(StructAccess<'a>),
//     TupleAccess(TupleAccess<'a>),
//     Tuple(Tuple<'a>),
//     Assignment(Assignment<'a>),
//     EscapeBlock(EscapeBlock<'a>),
//     MemberAccess(UnresolvedMemberAccess<'a>),
// }

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NodeRef<'a, T>
where
    T: IrArenaType<'a> + ?Sized,
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
    #[inline]
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
    #[inline]
    fn into(self) -> Id<T> {
        self.id
    }
}

pub struct IrNodeArena<'a> {
    pub stmt_blks: Arena<StmtBlock<'a>>,
    pub stmt: Arena<Stmt<'a>>,
    pub exprs: Arena<Expr<'a>>,
    pub func_args: Arena<FunctionArg<'a>>,
    pub st_attrs: Arena<StructAttr<'a>>,
    pub enms: Arena<Enum<'a>>,
    pub enm_vals: Arena<EnumValue<'a>>,
    pub enm_inits: Arena<EnumInit<'a>>,
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
    pub mem_accs: Arena<UnresolvedMemberAccess<'a>>,
}

impl<'a> IrNodeArena<'a> {
    pub fn new() -> Self {
        IrNodeArena {
            stmt_blks: Arena::new(),
            stmt: Arena::new(),
            exprs: Arena::new(),
            func_args: Arena::new(),
            st_attrs: Arena::new(),
            enms: Arena::new(),
            enm_vals: Arena::new(),
            enm_inits: Arena::new(),
            funcs: Arena::new(),
            func_calls: Arena::new(),
            st_decls: Arena::new(),
            st_init_vals: Arena::new(),
            st_inits: Arena::new(),
            st_accs: Arena::new(),
            tup_accs: Arena::new(),
            tups: Arena::new(),
            asgns: Arena::new(),
            esc_blks: Arena::new(),
            var_decls: Arena::new(),
            mem_accs: Arena::new(),
        }
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

impl<'a> IrArenaType<'a> for StmtBlock<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.stmt_blks
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.stmt_blks
    }
}

impl<'a> IrArenaType<'a> for Stmt<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.stmt
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.stmt
    }
}

impl<'a> IrArenaType<'a> for Expr<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.exprs
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.exprs
    }
}

impl<'a> IrArenaType<'a> for FunctionArg<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.func_args
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.func_args
    }
}

impl<'a> IrArenaType<'a> for StructAttr<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_attrs
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_attrs
    }
}

impl<'a> IrArenaType<'a> for Enum<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.enms
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.enms
    }
}

impl<'a> IrArenaType<'a> for EnumValue<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.enm_vals
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.enm_vals
    }
}

impl<'a> IrArenaType<'a> for EnumInit<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.enm_inits
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.enm_inits
    }
}

impl<'a> IrArenaType<'a> for Function<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.funcs
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.funcs
    }
}

impl<'a> IrArenaType<'a> for FunctionCall<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.func_calls
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.func_calls
    }
}

impl<'a> IrArenaType<'a> for Struct<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_decls
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_decls
    }
}

impl<'a> IrArenaType<'a> for StructInitValue<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_init_vals
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_init_vals
    }
}

impl<'a> IrArenaType<'a> for StructInit<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_inits
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_inits
    }
}

impl<'a> IrArenaType<'a> for StructAccess<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.st_accs
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.st_accs
    }
}

impl<'a> IrArenaType<'a> for Tuple<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.tups
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.tups
    }
}

impl<'a> IrArenaType<'a> for TupleAccess<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.tup_accs
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.tup_accs
    }
}

impl<'a> IrArenaType<'a> for Assignment<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.asgns
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.asgns
    }
}

impl<'a> IrArenaType<'a> for EscapeBlock<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.esc_blks
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.esc_blks
    }
}

impl<'a> IrArenaType<'a> for VarDecl<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.var_decls
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.var_decls
    }
}

impl<'a> IrArenaType<'a> for UnresolvedMemberAccess<'a> {
    #[inline]
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
        &ctx.nodes.mem_accs
    }

    #[inline]
    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
        &mut ctx.nodes.mem_accs
    }
}
