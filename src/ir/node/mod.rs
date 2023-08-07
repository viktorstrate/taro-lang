use id_arena::{Arena, Id};

use self::{
    assignment::Assignment,
    control_flow::IfStmt,
    enumeration::{Enum, EnumInit, EnumValue},
    escape_block::EscapeBlock,
    expression::Expr,
    external::ExternalObject,
    function::{Function, FunctionArg, FunctionCall},
    generics::GenericsDecl,
    member_access::UnresolvedMemberAccess,
    statement::{Stmt, StmtBlock, VarDecl},
    structure::{Struct, StructAccess, StructAttr, StructInit, StructInitValue},
    traits::{Trait, TraitFuncAttr},
    tuple::{Tuple, TupleAccess},
};
use std::{convert::Into, marker::PhantomData};

use super::context::{IrArenaType, IrCtx};

pub mod assignment;
pub mod control_flow;
pub mod enumeration;
pub mod escape_block;
pub mod expression;
pub mod external;
pub mod function;
pub mod generics;
pub mod identifier;
pub mod member_access;
pub mod module;
pub mod statement;
pub mod structure;
pub mod traits;
pub mod tuple;
pub mod type_signature;

#[derive(Debug, Eq, Hash)]
pub struct NodeRef<'a, T>
where
    T: IrArenaType<'a> + ?Sized,
{
    id: Id<T>,
    _marker: PhantomData<&'a str>,
}

impl<'a, T> PartialEq for NodeRef<'a, T>
where
    T: IrArenaType<'a>,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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

macro_rules! register_nodes {
    ( $( ( $node_name:ident, $node_type:ty ) ),* ) => {
        pub struct IrNodeArena<'a> {
            $(
                pub $node_name: Arena<$node_type>,
            )*
        }

        impl<'a> IrNodeArena<'a> {
            pub fn new() -> Self {
                IrNodeArena {
                    $(
                        $node_name: Arena::new(),
                    )*
                }
            }
        }

        $(
            impl<'a> IrArenaType<'a> for $node_type {
                #[inline]
                fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b Arena<Self> {
                    &ctx.nodes.$node_name
                }

                #[inline]
                fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut Arena<Self> {
                    &mut ctx.nodes.$node_name
                }
            }
        )*
    };
}

register_nodes![
    (stmt_blks, StmtBlock<'a>),
    (stmt, Stmt<'a>),
    (exprs, Expr<'a>),
    (func_args, FunctionArg<'a>),
    (st_attrs, StructAttr<'a>),
    (enms, Enum<'a>),
    (enm_vals, EnumValue<'a>),
    (enm_inits, EnumInit<'a>),
    (funcs, Function<'a>),
    (func_calls, FunctionCall<'a>),
    (st_decls, Struct<'a>),
    (st_init_vals, StructInitValue<'a>),
    (st_inits, StructInit<'a>),
    (st_accs, StructAccess<'a>),
    (tup_accs, TupleAccess<'a>),
    (tups, Tuple<'a>),
    (traits, Trait<'a>),
    (tr_attr, TraitFuncAttr<'a>),
    (asgns, Assignment<'a>),
    (esc_blks, EscapeBlock<'a>),
    (var_decls, VarDecl<'a>),
    (mem_accs, UnresolvedMemberAccess<'a>),
    (extern_obj, ExternalObject<'a>),
    (if_branch, IfStmt<'a>),
    (gen_decls, GenericsDecl<'a>)
];

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
