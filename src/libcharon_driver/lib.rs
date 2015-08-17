#![crate_name = "charon_driver"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]


extern crate syntax;

use syntax::ast;
use syntax::parse::parser::{Parser};

pub fn run(args: Vec<String>) -> i32{
	run_compiler();
	0
}

fn run_compiler(){
	let mut p = Parser::new("break".to_string());
	p.run();
}

#[test]
fn test_mate(){
}
