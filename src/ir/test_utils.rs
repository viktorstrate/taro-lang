#[cfg(test)]
pub mod utils {
    use crate::{
        code_gen::format_ir,
        ir::{
            ast_lowering::{lower_ast, LowerAstResult},
            ir_walker::walk_ir,
        },
        parser::{parse_ast, ParserError},
        symbols::{
            symbol_collector::SymbolCollector,
            symbol_resolver::SymbolResolver,
            symbol_table::{SymbolCollectionError, SymbolTable},
        },
        type_checker::{
            type_inference::TypeInferrer, type_resolver::TypeResolver, types_walker::TypeChecker,
            TypeCheckerError,
        },
        TranspilerError,
    };

    pub fn lowered_ir<'a>(input: &'a str) -> Result<LowerAstResult<'a>, ParserError<'a>> {
        let ast = parse_ast(input)?;
        Ok(lower_ast(ast))
    }

    pub fn collect_symbols<'a>(
        ir_result: &mut LowerAstResult<'a>,
    ) -> Result<SymbolTable<'a>, SymbolCollectionError<'a>> {
        let result = walk_ir(
            &mut SymbolCollector {},
            &mut ir_result.ctx,
            &mut ir_result.ir,
        );

        match result {
            Ok(val) => Ok(val),
            Err(err) => {
                match &err {
                    SymbolCollectionError::SymbolAlreadyExistsInScope(ident) => {
                        println!(
                            "SYMBOL ALREADY EXISTS IN SCOPE: {:?}",
                            ir_result.ctx[*ident]
                        )
                    }
                    _ => unreachable!(),
                }
                Err(err)
            }
        }
    }

    pub fn type_check<'a>(
        ir_result: &mut LowerAstResult<'a>,
    ) -> Result<TypeChecker<'a>, TypeCheckerError<'a>> {
        let symbols = collect_symbols(ir_result).unwrap();

        let ctx = &mut ir_result.ctx;
        let ir = &mut ir_result.ir;

        let mut sym_resolver = SymbolResolver::new(symbols);
        walk_ir(&mut sym_resolver, ctx, ir).unwrap();

        let mut type_inferrer = TypeInferrer::new(&ctx, sym_resolver);
        walk_ir(&mut type_inferrer, ctx, ir)?;

        let mut type_resolver = TypeResolver::new(ctx, type_inferrer);
        walk_ir(&mut type_resolver, ctx, ir)?;

        let mut checker = TypeChecker::new(ctx, type_resolver);
        let result = walk_ir(&mut checker, ctx, ir);

        match result {
            Ok(_val) => Ok(checker),
            Err(err) => Err(err),
        }
    }

    pub fn final_codegen<'a>(input: &'a str) -> Result<String, TranspilerError<'a>> {
        let ast = parse_ast(&input).map_err(TranspilerError::Parse)?;
        let mut lowered_ast = lower_ast(ast);

        let type_checker = type_check(&mut lowered_ast).unwrap();

        let ctx = &mut lowered_ast.ctx;
        let ir = &mut lowered_ast.ir;

        let mut buf = Vec::new();
        format_ir(&mut buf, ctx, type_checker.symbols, ir).map_err(TranspilerError::Write)?;
        let out = String::from_utf8(buf).unwrap();

        Ok(out)
    }
}
