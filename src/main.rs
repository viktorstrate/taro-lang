#![feature(map_try_insert)]
#![feature(iter_intersperse)]
#![feature(associated_type_defaults)]
#![feature(assert_matches)]
#![feature(let_else)]

use std::io::BufRead;

use code_gen::ast_to_js;
use parser::ParserError;
use symbols::symbol_table::SymbolsError;
use type_checker::{types_walker::TypeChecker, TypeCheckerError};

use crate::{
    ast::ast_walker::walk_ast, parser::parse_ast, symbols::symbol_walker::SymbolCollector,
};

pub mod ast;
pub mod code_gen;
pub mod formatter;
pub mod parser;
pub mod symbols;
pub mod type_checker;

fn main() -> std::io::Result<()> {
    let input = std::io::stdin()
        .lock()
        .lines()
        .collect::<Result<String, _>>()?;

    match transpile(&input) {
        Ok(output) => print!("{}", &output),
        Err(err) => println!("Error: {:?}", err),
    }

    Ok(())
}

#[derive(Debug)]
enum TranspilerError<'a> {
    Parse(ParserError<'a>),
    Symbols(SymbolsError<'a>),
    TypeCheck(TypeCheckerError<'a>),
}

fn transpile(input: &str) -> Result<String, TranspilerError> {
    let mut ast = parse_ast(&input).map_err(TranspilerError::Parse)?;

    let mut sym_collector = SymbolCollector::default();
    let sym_table = walk_ast(&mut sym_collector, &mut ast).map_err(TranspilerError::Symbols)?;

    let mut type_checker = TypeChecker::new(sym_table);
    walk_ast(&mut type_checker, &mut ast).map_err(TranspilerError::TypeCheck)?;

    Ok(ast_to_js(&ast))
}
