use crate::{
    error_message::error_formatter::Spanned,
    ir::context::IrCtx,
    ir::{late_init::LateInit, node::type_signature::TypeSignatureValue},
    parser::Span,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::StmtBlock,
    type_signature::{
        TypeEvalError, TypeSignature, TypeSignatureContext, TypeSignatureParent, Typed,
    },
    NodeRef,
};

#[derive(Debug)]
pub struct Function<'a> {
    pub name: LateInit<Ident<'a>>,
    pub args: Vec<NodeRef<'a, FunctionArg<'a>>>,
    pub return_type: LateInit<TypeSignature<'a>>,
    pub body: NodeRef<'a, StmtBlock<'a>>,
    pub span: Span<'a>,
}

impl<'a> NodeRef<'a, Function<'a>> {
    pub fn calculate_type_sig(
        &self,
        ctx: &mut IrCtx<'a>,
        // args: Vec<NodeRef<'a, FunctionArg<'a>>>,
        // return_type: TypeSignature<'a>,
    ) -> TypeSignature<'a> {
        let arg_types = ctx[*self]
            .args
            .clone()
            .into_iter()
            .map(|arg| &ctx[arg])
            .map(|arg| (*arg.type_sig).clone())
            .collect::<Vec<_>>();

        ctx.get_type_sig(
            TypeSignatureValue::Function {
                args: arg_types.into(),
                return_type: (*ctx[*self].return_type).clone().into(),
            },
            TypeSignatureContext {
                parent: TypeSignatureParent::Function(*self),
                type_span: None,
            }
            .alloc(),
        )
    }
}

#[derive(Debug)]
pub struct FunctionArg<'a> {
    pub name: LateInit<Ident<'a>>,
    pub type_sig: LateInit<TypeSignature<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub func: NodeRef<'a, Expr<'a>>,
    pub args: Vec<NodeRef<'a, Expr<'a>>>,
    pub args_span: Span<'a>,
}

impl<'a> Identifiable<'a> for Function<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.name
    }
}

impl<'a> Identifiable<'a> for FunctionArg<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        *self.name
    }
}

impl<'a> Spanned<'a> for NodeRef<'a, FunctionArg<'a>> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        Some(ctx[*self].span.clone())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, Function<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let args = ctx.nodes.funcs[self.id]
            .args
            .clone()
            .into_iter()
            .map(|arg| arg.eval_type(symbols, ctx))
            .collect::<Result<Vec<_>, _>>()?
            .into();

        Ok(ctx.get_type_sig(
            TypeSignatureValue::Function {
                args,
                return_type: (*ctx[*self].return_type).clone().into(),
            },
            TypeSignatureContext {
                parent: TypeSignatureParent::Function(*self),
                type_span: None,
            }
            .alloc(),
        ))
    }

    fn specified_type(&self, _ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        // Some(self.calculate_type_sig(ctx))
        None
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        let (new_args, new_return_type) = match &ctx[&new_type] {
            TypeSignatureValue::Function { args, return_type } => {
                (args.clone(), (*return_type).clone())
            }
            _ => unreachable!("specified type expected to be function"),
        };

        let func_args_len = ctx[*self].args.len();
        if new_args.len() != func_args_len {
            return Err(TypeEvalError::FuncWrongNumberOfArgs {
                func: *self,
                expected: new_args.len(),
                actual: func_args_len,
            });
        }

        for (arg_type, arg) in new_args.iter().zip(ctx[*self].args.clone().into_iter()) {
            ctx[arg].type_sig = (*arg_type).clone().into();
        }

        ctx[*self].return_type = new_return_type.into();

        Ok(())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, FunctionArg<'a>> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok((*ctx[*self].type_sig).clone())
    }

    fn specified_type(&self, ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        Some((*ctx[*self].type_sig).clone())
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        ctx[*self].type_sig = new_type.into();
        Ok(())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, FunctionCall<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let type_sig = ctx[*self].func.clone().eval_type(symbols, ctx)?;
        match &ctx[&type_sig] {
            TypeSignatureValue::Function {
                args: _,
                return_type,
            } => Ok((**return_type).clone()),
            _wrong_type => Err(TypeEvalError::CallNonFunction(type_sig)),
        }
    }
}
