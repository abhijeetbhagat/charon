#![crate_name = "syntax"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

pub mod parse;
pub mod ast;


pub mod syntax {
    pub use parse;
    pub use ast;
}

pub mod visit;