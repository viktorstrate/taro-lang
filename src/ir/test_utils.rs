#[cfg(test)]
pub mod utils {
    use crate::{
        ir::{
            ast_lowering::{lower_ast, LowerAstResult},
            ir_walker::walk_ir,
            IR,
        },
        parser::{parse_ast, ParserError},
        symbols::{symbol_collector::SymbolCollector, symbol_resolver::SymbolResolver},
        type_checker::{types_walker::TypeChecker, TypeCheckerError},
    };

    #[derive(Debug)]
    pub enum FinalIrError<'a> {
        Parser(ParserError<'a>),
        TypeCheck(TypeCheckerError<'a>),
    }

    pub fn lowered_ir<'a>(input: &'a str) -> Result<LowerAstResult<'a>, ParserError<'a>> {
        let ast = parse_ast(input)?;
        Ok(lower_ast(ast))
    }

    pub fn final_ir<'a>(input: &'a str) -> Result<IR<'a>, FinalIrError<'a>> {
        let mut result = lowered_ir(input).map_err(FinalIrError::Parser)?;

        type_check(&mut result).map_err(FinalIrError::TypeCheck)?;

        Ok(result.ir)
    }

    pub fn type_check<'a>(ir_result: &mut LowerAstResult<'a>) -> Result<(), TypeCheckerError<'a>> {
        let ctx = &mut ir_result.ctx;
        let ir = &mut ir_result.ir;

        let mut sym_collector = SymbolCollector {};
        let symbols = walk_ir(&mut sym_collector, ctx, ir).unwrap();

        let mut sym_resolver = SymbolResolver::new(symbols);
        walk_ir(&mut sym_resolver, ctx, ir).unwrap();

        let mut checker = TypeChecker::new(sym_resolver);
        return walk_ir(&mut checker, ctx, ir);
    }

    // pub fn final_codegen(input: &str) -> Result<String, FinalIrError> {
    //     let mut ast = parse_ast(input).map_err(FinalIrError::Parser)?;

    //     let mut sym_collector = SymbolCollector {};
    //     let symbols = walk_ast(&mut sym_collector, &mut ast).unwrap();

    //     let mut checker = TypeChecker::new(symbols);
    //     walk_ast(&mut checker, &mut ast).map_err(FinalIrError::TypeCheck)?;

    //     checker.symbols.reset();
    //     let mut buf = Vec::new();
    //     format_ast(&mut buf, &ast, checker.symbols).unwrap();

    //     let out = String::from_utf8(buf).unwrap();

    //     Ok(out)
    // }
}
