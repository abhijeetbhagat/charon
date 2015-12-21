
use ast::{Expr, Block, Stmt, Decl};

pub trait Visitor<'a> : Sized  {
    fn visit_block(&mut self, block : &'a Block){

    }

    fn visit_stmt(&mut self, stmt : &'a Stmt){

    }

    fn visit_expr(&mut self, expr : &'a mut Expr){
    }

    fn visit_decl(&mut self, decl : &'a mut Decl){

    }
}

