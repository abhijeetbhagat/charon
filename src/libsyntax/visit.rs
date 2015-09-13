
use ast::{Expr, Block, Stmt, Decl};

pub trait Visitor<'a> : Sized  {
    fn visit_block(&mut self, block : &'a Block){

    }

    fn visit_stmt(&mut self, stmt : &'a Stmt){

    }

    fn visit_expr(&mut self, expr : &'a Expr){
        walk_expr(self, expr)
    }

    fn visit_decl(&mut self, decl : &'a Decl){

    }
}

pub fn walk_expr<'a, V: Visitor<'a>>(visitor : &mut V, expr: &'a Expr){
    match expr{
        &Expr::AddExpr(ref left_expr, ref right_expr) =>{
            visitor.visit_expr(left_expr);
            visitor.visit_expr(right_expr);
        },
        &Expr::NumExpr(value) => {

        }
        _ => {}
    }
}
