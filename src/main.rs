#![feature(map_try_insert)]
#![feature(iter_intersperse)]
#![feature(associated_type_defaults)]
#![feature(assert_matches)]
#![feature(let_else)]

use parser::ParserError;
use std::io::{BufRead, Write};
use symbols::symbol_table::SymbolsError;
use type_checker::{types_walker::TypeChecker, TypeCheckerError};

use crate::{
    ast::ast_walker::walk_ast, code_gen::format_ast, parser::parse_ast,
    symbols::symbol_walker::SymbolCollector,
};

pub mod ast;
pub mod code_gen;
pub mod parser;
pub mod symbols;
pub mod type_checker;

fn main() -> std::io::Result<()> {
    let mut input = std::io::stdin()
        .lock()
        .lines()
        .collect::<Result<Vec<String>, _>>()?;

    input.iter_mut().for_each(|line| *line += "\n");
    let input = input.into_iter().collect::<String>();

    if let Err(err) = transpile(std::io::stdout(), &input) {
        println!("Error: {:?}", err)
    }

    Ok(())
}

#[derive(Debug)]
enum TranspilerError<'a> {
    Parse(ParserError<'a>),
    Symbols(SymbolsError<'a>),
    TypeCheck(TypeCheckerError<'a>),
    Write(std::io::Error),
}

fn transpile<W: Write>(writer: W, input: &str) -> Result<(), TranspilerError> {
    let mut ast = parse_ast(&input).map_err(TranspilerError::Parse)?;

    let mut sym_collector = SymbolCollector::default();
    let sym_table = walk_ast(&mut sym_collector, &mut ast).map_err(TranspilerError::Symbols)?;

    let mut type_checker = TypeChecker::new(sym_table);
    walk_ast(&mut type_checker, &mut ast).map_err(TranspilerError::TypeCheck)?;

    format_ast(writer, &ast, type_checker.symbols).map_err(TranspilerError::Write)?;

    Ok(())
}
