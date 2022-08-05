use id_arena::{Arena};

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
