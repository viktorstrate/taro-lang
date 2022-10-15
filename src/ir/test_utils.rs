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
        type_checker::{TypeChecker, TypeCheckerError},
        TranspilerError,
    };

    pub fn lowered_ir<'a>(input: &'a str) -> Result<LowerAstResult<'a>, ParserError<'a>> {
        let ast = parse_ast(input)?;
        Ok(lower_ast(ast))
    }

    pub fn collect_symbols<'a>(
        ir_result: &mut LowerAstResult<'a>,
    ) -> Result<SymbolTable<'a>, SymbolCollectionError<'a>> {
        let result = walk_ir(&mut SymbolCollector {}, ir_result);

        match result {
            Ok(val) => Ok(val),
            Err(err) => {
                match &err {
                    SymbolCollectionError::SymbolAlreadyExistsInScope {
                        new: ident,
                        existing: _,
                    } => {
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

        let mut sym_resolver = SymbolResolver::new(symbols);
        walk_ir(&mut sym_resolver, ir_result).unwrap();

        let mut type_checker = TypeChecker::new(&mut ir_result.ctx, sym_resolver);
        let result = type_checker.type_check(ir_result);

        match result {
            Ok(_) => Ok(type_checker),
            Err(err) => Err(err),
        }
    }

    pub fn final_codegen<'a>(input: &'a str) -> Result<String, TranspilerError<'a>> {
        let ast = parse_ast(&input).map_err(TranspilerError::Parse)?;
        let mut lowered_ast = lower_ast(ast);

        let type_checker = match type_check(&mut lowered_ast) {
            Ok(type_checker) => type_checker,
            Err(err) => panic!("TYPE CHECK ERROR: {:?}", err),
        };

        let ctx = &mut lowered_ast.ctx;
        let ir = &mut lowered_ast.ir;

        let mut buf = Vec::new();
        format_ir(&mut buf, ctx, type_checker.symbols, ir).map_err(TranspilerError::Write)?;
        let out = String::from_utf8(buf).unwrap();

        Ok(out)
    }
}
