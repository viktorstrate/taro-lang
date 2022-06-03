#![feature(map_try_insert)]
#![feature(iter_intersperse)]
#![feature(associated_type_defaults)]
#![feature(assert_matches)]
#![feature(let_else)]

pub mod ast;
pub mod code_gen;
pub mod formatter;
pub mod parser;
pub mod symbols;
pub mod type_checker;

fn main() {
    println!("Hello, world!");
}
