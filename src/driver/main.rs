/*
lua compiler:
main->load->parser->lex scan->chunk->stat->if/while/do/etc.
*/

#![allow(dead_code)]
#![allow(unused_imports)]
//use tokens;
//use lexer;
extern crate charon_driver as this;

use std::process;
use std::env;

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
    let result = this::run(env::args().collect());
    process::exit(result);
}
