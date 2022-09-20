#![feature(map_try_insert)]
#![feature(iter_intersperse)]
#![feature(associated_type_defaults)]
#![feature(assert_matches)]
#![feature(let_else)]
#![feature(hash_set_entry)]
#![deny(rust_2018_idioms)]
#![feature(iter_advance_by)]

use std::io::{BufRead, Write};

use code_gen::format_ir;
use error_message::ErrorMessage;
use ir::{
    ast_lowering::{lower_ast, LowerAstResult},
    ir_walker::walk_ir,
};
use parser::{parse_ast, ParserError};
use symbols::{
    symbol_collector::SymbolCollector,
    symbol_resolver::{SymbolResolutionError, SymbolResolver},
    symbol_table::SymbolCollectionError,
};
use type_checker::{TypeChecker, TypeCheckerError};

pub mod ast;
pub mod code_gen;
pub mod error_message;
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
        err.format_err(&mut std::io::stderr(), ())?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum TranspilerError<'a> {
    Parse(ParserError<'a>),
    SymbolCollectError(LowerAstResult<'a>, SymbolCollectionError<'a>),
    SymbolResolveError(LowerAstResult<'a>, SymbolResolutionError<'a>),
    TypeCheck(TypeChecker<'a>, LowerAstResult<'a>, TypeCheckerError<'a>),
    Write(std::io::Error),
}

fn transpile<'a, W: Write>(writer: &mut W, input: &'a str) -> Result<(), TranspilerError<'a>> {
    let ast = parse_ast(&input).map_err(TranspilerError::Parse)?;
    let mut la = lower_ast(ast);

    let sym_table = match walk_ir(&mut SymbolCollector {}, &mut la) {
        Ok(sym) => sym,
        Err(err) => return Err(TranspilerError::SymbolCollectError(la, err)),
    };

    let mut sym_resolver = SymbolResolver::new(sym_table);
    match walk_ir(&mut sym_resolver, &mut la) {
        Ok(_) => {}
        Err(err) => return Err(TranspilerError::SymbolResolveError(la, err)),
    }

    let mut type_checker = TypeChecker::new(&mut la.ctx, sym_resolver);
    match type_checker.type_check(&mut la) {
        Ok(_) => {}
        Err(err) => return Err(TranspilerError::TypeCheck(type_checker, la, err)),
    }

    match format_ir(writer, &mut la.ctx, type_checker.symbols, &mut la.ir) {
        Ok(_) => {}
        Err(err) => return Err(TranspilerError::Write(err)),
    }

    Ok(())
}
