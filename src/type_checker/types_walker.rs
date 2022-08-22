use crate::{
    ir::{
        context::IrCtx,
        ir_walker::{IrWalker, ScopeValue},
        node::{
            expression::Expr,
            statement::Stmt,
            NodeRef,
        },
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

use super::{
    assignment::check_assignment, struct_type::check_struct_init, type_resolver::TypeResolver,
    TypeCheckerError,
};

#[derive(Debug)]
pub struct TypeChecker<'a> {
    pub symbols: SymbolTableZipper<'a>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(ctx: &IrCtx<'a>, type_resolver: TypeResolver<'a>) -> Self {
        let mut symbols = type_resolver.symbols;
        symbols.reset(ctx);
        TypeChecker { symbols }
    }
}

impl<'a> IrWalker<'a> for TypeChecker<'a> {
    type Error = TypeCheckerError<'a>;

    fn visit_scope_begin(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        value: ScopeValue<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        value.visit_scope_begin(ctx, &mut self.symbols);
        Ok(())
    }

    fn visit_scope_end(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _parent: &mut Self::Scope,
        _child: Self::Scope,
        _value: ScopeValue<'a>,
    ) -> Result<(), TypeCheckerError<'a>> {
        self.symbols
            .exit_scope(ctx)
            .expect("scope should not be global scope");

        Ok(())
    }

    fn visit_ordered_symbol(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
    ) -> Result<(), Self::Error> {
        self.symbols.visit_next_symbol(ctx);
        Ok(())
    }

    // fn pre_visit_stmt(
    //     &mut self,
    //     ctx: &mut IrCtx<'a>,
    //     _scope: &mut Self::Scope,
    //     stmt: NodeRef<'a, Stmt<'a>>,
    // ) -> Result<(), Self::Error> {
    //     match ctx[stmt] {
    //         Stmt::VariableDecl(var_decl) => match ctx[ctx[var_decl].value] {
    //             Expr::Function(func) => match ctx[var_decl].type_sig {
    //                 Some(type_sig) => match &ctx[type_sig] {
    //                     TypeSignatureValue::Function {
    //                         args: _,
    //                         return_type: _,
    //                     } => {
    //                         func.specify_type(ctx, type_sig)
    //                             .map_err(TypeCheckerError::TypeEvalError)?;
    //                     }
    //                     _ => {
    //                         return Err(TypeCheckerError::TypeSignatureMismatch {
    //                             type_sig,
    //                             expr_type: ctx.get_type_sig(TypeSignatureValue::Function {
    //                                 args: vec![],
    //                                 return_type: ctx.get_builtin_type_sig(BuiltinType::Void),
    //                             }),
    //                         });
    //                     }
    //                 },
    //                 None => {}
    //             },
    //             _ => {}
    //         },
    //         _ => {}
    //     }

    //     Ok(())
    // }

    fn visit_stmt(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut Self::Scope,
        stmt: NodeRef<'a, Stmt<'a>>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match ctx[stmt].clone() {
            // Stmt::VariableDecl(var_decl) => type_check(ctx, &mut self.symbols, var_decl),
            // Stmt::FunctionDecl(func_decl) => type_check(ctx, &mut self.symbols, func_decl),
            // Stmt::StructDecl(st) => {
            //     for attr in ctx[st].attrs.clone() {
            //         type_check(ctx, &mut self.symbols, attr)?;
            //     }
            //     Ok(())
            // }
            // Stmt::EnumDecl(enm) => {
            //     type_check(ctx, &mut self.symbols, enm)?;
            //     Ok(())
            // }
            _ => Ok(()),
        }
    }

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut (),
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match ctx[expr].clone() {
            // Expr::FunctionCall(call) => {
            //     let type_sig = ctx[call]
            //         .func
            //         .clone()
            //         .eval_type(&mut self.symbols, ctx)
            //         .map_err(TypeCheckerError::TypeEval)?;

            //     let (args, return_type) = match &ctx[type_sig] {
            //         TypeSignatureValue::Function { args, return_type } => {
            //             Ok((args.clone(), *return_type))
            //         }
            //         _ => Err(TypeCheckerError::CallNonFunction {
            //             ident_type: type_sig,
            //         }),
            //     }?;

            //     let param_types = ctx[call]
            //         .params
            //         .clone()
            //         .into_iter()
            //         .map(|param| param.eval_type(&mut self.symbols, ctx).unwrap())
            //         .collect::<Vec<_>>();

            //     let arg_count_match = ctx[call].params.len() == args.len();
            //     let args_match = param_types.iter().zip(args.iter()).all(|(a, b)| *a == *b);

            //     if !arg_count_match || !args_match {
            //         return Err(TypeCheckerError::TypeSignatureMismatch {
            //             type_sig: ctx
            //                 .get_type_sig(TypeSignatureValue::Function { args, return_type }),
            //             expr_type: ctx.get_type_sig(TypeSignatureValue::Function {
            //                 args: param_types,
            //                 return_type,
            //             }),
            //         });
            //     }

            //     Ok(())
            // }
            // Expr::Function(func) => type_check(ctx, &mut self.symbols, func),
            Expr::Assignment(asg) => check_assignment(ctx, &mut self.symbols, asg),
            Expr::StructInit(st_init) => check_struct_init(ctx, &mut self.symbols, st_init),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::test_utils::utils::{lowered_ir, type_check};

    #[test]
    fn test_var_decl_matching_types() {
        let mut ir = lowered_ir("let x: String = \"hello\"").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_));
    }

    #[test]
    fn test_struct_init_default() {
        let mut ir = lowered_ir(
            "\
        struct Test { let default = 34; let noDefault: Number }
        let test = Test { noDefault: 123 }",
        )
        .unwrap();

        assert!(type_check(&mut ir).is_ok())
    }

    #[test]
    fn test_struct_init_not_default() {
        let mut ir = lowered_ir(
            "\
        struct Test { let noDefault: Number }
        let test = Test {}",
        )
        .unwrap();

        assert!(type_check(&mut ir).is_err());
    }

    #[test]
    fn test_escape_block_function_return() {
        let mut ir = lowered_ir("func f() -> Number { return @{ 1 + 2 } }").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_));
    }

    #[test]
    fn test_escape_block_function_return_coerce() {
        let mut ir = lowered_ir("func f() -> Number { return @{ 1 + 2 }; return 2 }").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_));
    }
}
