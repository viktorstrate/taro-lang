use id_arena::Id;

use crate::{
    ir::context::IrCtx,
    ir::node::type_signature::TypeSignatureValue,
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
    type_checker::function_body_type_eval::{eval_func_body_type_sig, FunctionTypeError},
};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    statement::Stmt,
    type_signature::{TypeEvalError, TypeSignature, Typed},
    NodeRef,
};

#[derive(Debug)]
pub struct Function<'a> {
    pub name: Ident<'a>,
    pub args: Vec<NodeRef<'a, FunctionArg<'a>>>,
    pub return_type: Option<TypeSignature<'a>>,
    pub body: NodeRef<'a, Stmt<'a>>,
}

impl<'a> Function<'a> {
    pub fn calculate_type_sig(
        ctx: &mut IrCtx<'a>,
        args: Vec<NodeRef<'a, FunctionArg<'a>>>,
        return_type: Option<TypeSignature<'a>>,
    ) -> Option<TypeSignature<'a>> {
        let arg_types = args
            .into_iter()
            .map(|arg| &ctx[arg])
            .map(|arg| arg.type_sig)
            .collect::<Option<Vec<_>>>();

        match (arg_types, return_type) {
            (Some(args), Some(return_type)) => {
                Some(ctx.get_type_sig(TypeSignatureValue::Function { args, return_type }))
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub type_sig: Option<TypeSignature<'a>>,
}

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub func: NodeRef<'a, Expr<'a>>,
    pub params: Vec<NodeRef<'a, Expr<'a>>>,
}

impl<'a> Identifiable<'a> for Function<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

impl<'a> Identifiable<'a> for FunctionArg<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
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
            .iter()
            .map(|arg| arg.eval_type(symbols, ctx))
            .collect::<Result<Vec<_>, _>>()?;

        symbols
            .enter_scope(ctx, ctx[*self].name)
            .expect("function should be located in current scope");

        let return_type =
            eval_func_body_type_sig(ctx, symbols, *self).map_err(TypeEvalError::FunctionType)?;

        symbols.exit_scope(ctx).unwrap();

        Ok(ctx.get_type_sig(TypeSignatureValue::Function {
            args,
            return_type: return_type,
        }))
    }

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        Function::calculate_type_sig(ctx, ctx[*self].args.clone(), ctx[*self].return_type)
    }

    fn specify_type(
        &mut self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        let TypeSignatureValue::Function { args, return_type: _ } = &ctx[new_type] else {
            unreachable!("specified type expected to be function");
        };

        let func_args_len = ctx[*self].args.len();
        if args.len() != func_args_len {
            return Err(TypeEvalError::FunctionType(
                FunctionTypeError::WrongNumberOfArgs {
                    func: *self,
                    expected: args.len(),
                    actual: func_args_len,
                },
            ));
        }

        for (arg_type, arg) in args.iter().zip(ctx[*self].args.into_iter()) {
            ctx[arg].type_sig = Some(*arg_type);
        }

        Ok(())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, FunctionArg<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let arg = &ctx[*self];
        arg.type_sig
            .ok_or(TypeEvalError::UndeterminableType(arg.name))
    }

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        ctx[*self].type_sig
    }

    fn specify_type(
        &mut self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        ctx[*self].type_sig = Some(new_type);
        Ok(())
    }
}

impl<'a> Typed<'a> for NodeRef<'a, FunctionCall<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let type_sig = ctx[*self].func.eval_type(symbols, ctx)?;
        match &ctx[type_sig] {
            TypeSignatureValue::Function {
                args: _,
                return_type,
            } => Ok(*return_type),
            _wrong_type => Err(TypeEvalError::CallNonFunction(type_sig)),
        }
    }
}
