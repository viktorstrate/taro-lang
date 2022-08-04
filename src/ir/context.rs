use std::{cell::Cell, collections::HashMap};

use typed_arena::Arena;

use crate::ast;

use super::node::{
    assignment::Assignment,
    enumeration::EnumValue,
    escape_block::EscapeBlock,
    expression::Expr,
    function::{Function, FunctionArg, FunctionCall},
    identifier::{Ident, IdentValue, ResolvedIdentValue},
    statement::Stmt,
    structure::{StructAccess, StructAttr, StructInit, StructInitValue},
    tuple::{Tuple, TupleAccess},
    type_signature::{BuiltinType, TypeSignature, TypeSignatureValue},
    IrNode,
};

pub struct IrCtx<'a, 'ctx> {
    types: Arena<Cell<&'ctx TypeSignatureValue<'a, 'ctx>>>,
    type_vals: Arena<TypeSignatureValue<'a, 'ctx>>,
    types_lookup:
        HashMap<crate::ast::node::type_signature::TypeSignatureValue<'a>, TypeSignature<'a, 'ctx>>,
    idents: Arena<Cell<&'ctx IdentValue<'a>>>,
    ident_vals: Arena<IdentValue<'a>>,
    nodes: Arena<IrNode<'a, 'ctx>>,
}

impl<'a, 'ctx> IrCtx<'a, 'ctx> {
    pub fn get_type_sig(
        &'ctx self,
        type_sig: crate::ast::node::type_signature::TypeSignature<'a>,
    ) -> TypeSignature<'a, 'ctx> {
        if let Some(found_type) = self.types_lookup.get(&type_sig.value) {
            return *found_type;
        }

        let ir_type = match type_sig.value {
            ast::node::type_signature::TypeSignatureValue::Base(base) => {
                TypeSignatureValue::Unresolved(base)
            }
            ast::node::type_signature::TypeSignatureValue::Function { args, return_type } => {
                TypeSignatureValue::Function {
                    args: args.into_iter().map(|arg| self.get_type_sig(arg)).collect(),
                    return_type: self.get_type_sig(*return_type),
                }
            }
            ast::node::type_signature::TypeSignatureValue::Tuple(types) => {
                TypeSignatureValue::Tuple(types.into_iter().map(|t| self.get_type_sig(t)).collect())
            }
        };

        let type_val = self.type_vals.alloc(ir_type);

        TypeSignature(self.types.alloc(Cell::new(type_val)))
    }

    pub fn make_ident(&'ctx self, ident: ast::node::identifier::Ident<'a>) -> Ident<'a, 'ctx> {
        let value = self
            .ident_vals
            .alloc(IdentValue::Resolved(ResolvedIdentValue::Named {
                def_span: ident.span,
                name: ident.value,
            }));

        let ident = self.idents.alloc(Cell::new(value));
        Ident(ident)
    }

    pub fn make_builtin_ident(&'ctx self, builtin: BuiltinType) -> Ident<'a, 'ctx> {
        let value = self
            .ident_vals
            .alloc(IdentValue::Resolved(ResolvedIdentValue::BuiltinType(
                builtin,
            )));

        let ident = self.idents.alloc(Cell::new(value));
        Ident(ident)
    }

    pub fn make_anon_ident(&'ctx self) -> Ident<'a, 'ctx> {
        let value = self
            .ident_vals
            .alloc(IdentValue::Resolved(ResolvedIdentValue::Anonymous));

        let ident = self.idents.alloc(Cell::new(value));
        Ident(ident)
    }

    pub fn make_unresolved_ident(
        &'ctx self,
        ident: ast::node::identifier::Ident<'a>,
    ) -> Ident<'a, 'ctx> {
        let value = self.ident_vals.alloc(IdentValue::Unresolved(ident));

        let ident = self.idents.alloc(Cell::new(value));
        Ident(ident)
    }
}

pub trait IrAllocate<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self;
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for Stmt<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::Stmt(self));

        match node {
            IrNode::Stmt(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for Expr<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::Expr(self));

        match node {
            IrNode::Expr(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for FunctionArg<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::FunctionArg(self));

        match node {
            IrNode::FunctionArg(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for StructAttr<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::StructAttr(self));

        match node {
            IrNode::StructAttr(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for EnumValue<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::EnumValue(self));

        match node {
            IrNode::EnumValue(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for Function<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::Function(self));

        match node {
            IrNode::Function(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for FunctionCall<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::FunctionCall(self));

        match node {
            IrNode::FunctionCall(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for StructInitValue<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::StructInitValue(self));

        match node {
            IrNode::StructInitValue(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for StructInit<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::StructInit(self));

        match node {
            IrNode::StructInit(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for StructAccess<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::StructAccess(self));

        match node {
            IrNode::StructAccess(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for TupleAccess<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::TupleAccess(self));

        match node {
            IrNode::TupleAccess(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for Tuple<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::Tuple(self));

        match node {
            IrNode::Tuple(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for Assignment<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::Assignment(self));

        match node {
            IrNode::Assignment(module) => module,
            _ => unreachable!(),
        }
    }
}

impl<'a, 'ctx> IrAllocate<'a, 'ctx> for EscapeBlock<'a, 'ctx> {
    fn allocate(self, ctx: &'ctx IrCtx<'a, 'ctx>) -> &'ctx mut Self {
        let node = ctx.nodes.alloc(IrNode::EscapeBlock(self));

        match node {
            IrNode::EscapeBlock(module) => module,
            _ => unreachable!(),
        }
    }
}
