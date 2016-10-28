#![crate_name = "syntax"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]
#![feature(convert)]
#![feature(plugin)]
//#![plugin(clippy)]
#[macro_use]extern crate itertools;

pub mod parse;
pub mod ast;
pub mod ptr;


pub mod syntax {
    pub use parse;
    pub use ast;
}

pub mod visit;
pub mod visitor_impl;
