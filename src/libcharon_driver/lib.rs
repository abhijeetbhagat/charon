#![crate_name = "charon_driver"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

extern crate syntax;
extern crate trans;

use syntax::ast;
use syntax::parse::parser::{Parser};
use trans::base::translate;
pub fn run(args: Vec<String>) -> i32{
	run_compiler();
	0
}

fn run_compiler(){
	let mut p = Parser::new("break".to_string());
	let optional_blk = p.run();
    if optional_blk.is_none(){
        panic!("Could not compile");
    }
   
	//TODO extract the expr and call pass it to trans to get the llvm context
    translate(&*optional_blk.unwrap().expr.unwrap());
}

#[test]
fn test_mate(){
}
