/*
lua compiler:
main->load->parser->lex scan->chunk->stat->if/while/do/etc.
*/

#![allow(dead_code)]
#![allow(unused_imports)]
//use tokens;
//use lexer;
mod lexer;
mod tokens;
mod parser;
mod lua_types;
mod statements;
mod expressions;
mod block;
mod traits;
use lua_types::*;
use lexer::*;
use tokens::*;
use statements::*;
use expressions::*;
use parser::*;
use block::*;
use traits::*;

trait SourceReader{
    fn get_src()->String;
}

struct TestReader;
struct FileReader;

impl SourceReader for TestReader{
    fn get_src()->String{
        "1+1".to_string()
    }
} 

fn main() {
    let mut p = Parser::new("a".to_string());
    println!("Hello, world!");
}
