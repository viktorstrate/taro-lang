#[cfg(test)]
pub mod utils {
    use crate::{
        code_gen::format_ast,
        ir::{ast_walker::walk_ast, AST},
        parser::{parse_ast, ParserError},
        symbols::symbol_walker::SymbolCollector,
        type_checker::{types_walker::TypeChecker, TypeCheckerError},
    };

    #[derive(Debug)]
    pub enum FinalAstError<'a> {
        Parser(ParserError<'a>),
        TypeCheck(TypeCheckerError<'a>),
    }

    pub fn final_ast<'a>(input: &'a str) -> Result<AST<'a>, FinalAstError<'a>> {
        let mut ast = parse_ast(input).map_err(FinalAstError::Parser)?;
        type_check(&mut ast).map_err(FinalAstError::TypeCheck)?;

        Ok(ast)
    }

    pub fn type_check<'a>(ast: &mut AST<'a>) -> Result<(), TypeCheckerError<'a>> {
        let mut sym_collector = SymbolCollector {};
        let symbols = walk_ast(&mut sym_collector, ast).unwrap();

        let mut checker = TypeChecker::new(symbols);
        return walk_ast(&mut checker, ast);
    }

    pub fn final_codegen(input: &str) -> Result<String, FinalAstError> {
        let mut ast = parse_ast(input).map_err(FinalAstError::Parser)?;

        let mut sym_collector = SymbolCollector {};
        let symbols = walk_ast(&mut sym_collector, &mut ast).unwrap();

        let mut checker = TypeChecker::new(symbols);
        walk_ast(&mut checker, &mut ast).map_err(FinalAstError::TypeCheck)?;

        checker.symbols.reset();
        let mut buf = Vec::new();
        format_ast(&mut buf, &ast, checker.symbols).unwrap();

        let out = String::from_utf8(buf).unwrap();

        Ok(out)
    }
}
