

/*pub trait SymbolVisitor{
	fn visit_num(&self, &NumExpression);
	fn visit_ident(&self, &IdentExpression);
}*/
use ast::{Expr}; 

/*pub trait Visitor<T : ?Sized> {
    fn visit(&mut self, t: T);
}

pub trait Accept{
    type Visitable = Expression;
    fn accept<'a>(&'a self, &mut Visitor<&'a Self::Visitable>);
}*/

//struct CodeGenVisitor;
/*fn main() {
    let visitor = PrintVisitor;
    
    visitor.visit(1);
    visitor.visit("Hello, world");
    visitor.visit(NotDisplayable);
}*/

pub trait Visitor<'a> : Sized  {
    fn visit_expr(&mut self, expr : &'a Expr){
        walk_expr(self, expr)
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

