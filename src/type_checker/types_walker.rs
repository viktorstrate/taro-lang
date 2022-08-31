use crate::{
    ir::{
        context::IrCtx,
        ir_walker::{IrWalker, ScopeValue},
        node::{expression::Expr, NodeRef},
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

    fn visit_expr(
        &mut self,
        ctx: &mut IrCtx<'a>,
        _scope: &mut (),
        expr: NodeRef<'a, Expr<'a>>,
    ) -> Result<(), TypeCheckerError<'a>> {
        match ctx[expr].clone() {
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

        assert_matches!(type_check(&mut ir), Ok(_))
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

    #[test]
    fn test_decl_inside_scope() {
        let mut ir = lowered_ir("let f = () -> Boolean { let a = true; return a }").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_))
    }

    #[test]
    fn test_func_type_deduce() {
        let mut ir = lowered_ir("let f: (Boolean) -> Boolean = (val) { return val }").unwrap();
        assert_matches!(type_check(&mut ir), Ok(_))
    }
}
