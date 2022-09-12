#![feature(map_try_insert)]
#![feature(iter_intersperse)]
#![feature(associated_type_defaults)]
#![feature(assert_matches)]
#![feature(let_else)]
#![feature(hash_set_entry)]
#![deny(rust_2018_idioms)]

use std::io::{BufRead, Write};

use code_gen::format_ir;
use ir::{ast_lowering::lower_ast, ir_walker::walk_ir};
use parser::{parse_ast, ParserError};
use symbols::{
    symbol_collector::SymbolCollector,
    symbol_resolver::{SymbolResolutionError, SymbolResolver},
    symbol_table::SymbolCollectionError,
};
use type_checker::{TypeChecker, TypeCheckerError};

pub mod ast;
pub mod code_gen;
pub mod ir;
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

    if let Err(err) = transpile(&mut std::io::stdout(), &input) {
        println!("Error: {:?}", err)
    }

    Ok(())
}

#[derive(Debug)]
pub enum TranspilerError<'a> {
    Parse(ParserError<'a>),
    SymbolCollectError(SymbolCollectionError<'a>),
    SymbolResolveError(SymbolResolutionError<'a>),
    TypeCheck(TypeCheckerError<'a>),
    Write(std::io::Error),
}

fn transpile<'a, W: Write>(writer: &mut W, input: &'a str) -> Result<(), TranspilerError<'a>> {
    let ast = parse_ast(&input).map_err(TranspilerError::Parse)?;
    let mut lowered_ast = lower_ast(ast);

    let ctx = &mut lowered_ast.ctx;
    let ir = &mut lowered_ast.ir;

    let sym_table =
        walk_ir(&mut SymbolCollector {}, ctx, ir).map_err(TranspilerError::SymbolCollectError)?;

    let mut sym_resolver = SymbolResolver::new(sym_table);
    walk_ir(&mut sym_resolver, ctx, ir).map_err(TranspilerError::SymbolResolveError)?;

    let mut type_checker = TypeChecker::new(ctx, sym_resolver);
    type_checker
        .type_check(ctx, ir)
        .map_err(TranspilerError::TypeCheck)?;

    format_ir(writer, ctx, type_checker.symbols, ir).map_err(TranspilerError::Write)?;

    Ok(())
}
