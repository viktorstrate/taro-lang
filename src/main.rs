#![feature(map_try_insert)]
#![feature(iter_intersperse)]
#![feature(associated_type_defaults)]
#![feature(assert_matches)]

pub mod ast;
pub mod formatter;
pub mod parser;
pub mod symbols;
pub mod type_checker;

fn main() {
    println!("Hello, world!");
}
