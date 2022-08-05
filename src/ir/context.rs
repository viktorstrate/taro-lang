use std::{collections::HashMap};

use id_arena::Arena;

use crate::{
    ast,
    symbols::symbol_table::{SymbolValue, SymbolValueItem},
};

use super::node::{
    identifier::{Ident, IdentValue, ResolvedIdentValue},
    type_signature::{BuiltinType, TypeSignature, TypeSignatureValue}, IrNodeArena,
};

pub struct IrCtx<'a> {
    pub types: Arena<TypeSignatureValue<'a>>,
    types_lookup:
        HashMap<crate::ast::node::type_signature::TypeSignatureValue<'a>, TypeSignature<'a>>,
    pub idents: Arena<IdentValue<'a>>,
    pub nodes: IrNodeArena<'a>,
    pub symbols: Arena<SymbolValueItem<'a>>,
}

impl<'a> IrCtx<'a> {
    pub fn get_type_sig(
        &mut self,
        type_sig: crate::ast::node::type_signature::TypeSignature<'a>,
    ) -> TypeSignature<'a> {
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

        self.types.alloc(ir_type)
    }

    pub fn make_ident(&mut self, ident: ast::node::identifier::Ident<'a>) -> Ident<'a> {
        self.idents
            .alloc(IdentValue::Resolved(ResolvedIdentValue::Named {
                def_span: ident.span,
                name: ident.value,
            }))
    }

    pub fn make_builtin_ident(&mut self, builtin: BuiltinType) -> Ident<'a> {
        self.idents
            .alloc(IdentValue::Resolved(ResolvedIdentValue::BuiltinType(
                builtin,
            )))
    }

    pub fn make_anon_ident(&mut self) -> Ident<'a> {
        self.idents
            .alloc(IdentValue::Resolved(ResolvedIdentValue::Anonymous))
    }

    pub fn make_unresolved_ident(&mut self, ident: ast::node::identifier::Ident<'a>) -> Ident<'a> {
        self.idents.alloc(IdentValue::Unresolved(ident))
    }

    pub fn make_symbol(&mut self, symbol: SymbolValueItem<'a>) -> SymbolValue<'a> {
        self.symbols.alloc(symbol)
    }
}

// pub trait IrAllocate<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self;
// }

// impl<'a> IrAllocate<'a> for Stmt<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::Stmt(self));

//         match node {
//             IrNode::Stmt(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for Expr<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::Expr(self));

//         match node {
//             IrNode::Expr(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for FunctionArg<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::FunctionArg(self));

//         match node {
//             IrNode::FunctionArg(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for StructAttr<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::StructAttr(self));

//         match node {
//             IrNode::StructAttr(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for EnumValue<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::EnumValue(self));

//         match node {
//             IrNode::EnumValue(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for Function<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::Function(self));

//         match node {
//             IrNode::Function(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for FunctionCall<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::FunctionCall(self));

//         match node {
//             IrNode::FunctionCall(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for StructInitValue<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::StructInitValue(self));

//         match node {
//             IrNode::StructInitValue(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for StructInit<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::StructInit(self));

//         match node {
//             IrNode::StructInit(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for StructAccess<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::StructAccess(self));

//         match node {
//             IrNode::StructAccess(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for TupleAccess<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::TupleAccess(self));

//         match node {
//             IrNode::TupleAccess(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for Tuple<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::Tuple(self));

//         match node {
//             IrNode::Tuple(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for Assignment<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::Assignment(self));

//         match node {
//             IrNode::Assignment(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }

// impl<'a> IrAllocate<'a> for EscapeBlock<'a> {
//     fn allocate(self, ctx: &IrCtx<'a>) -> &mut Self {
//         let node = ctx.nodes.alloc(IrNode::EscapeBlock(self));

//         match node {
//             IrNode::EscapeBlock(module) => module,
//             _ => unreachable!(),
//         }
//     }
// }
