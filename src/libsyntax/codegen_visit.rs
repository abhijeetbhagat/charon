use ast::*;
use visit::Visitor;

/*impl<T> Visitor<T> for CodeGenVisitor where T: std::fmt::Display {
    fn visit(&self, t: T) {
        println!("{}", t);
    }
}*/

struct CodeGenVisitor;

impl<'a> Visitor<&'a NumExpression> for CodeGenVisitor {
    fn visit(&self, _: &'a NumExpression) {
        println!("Num expression visited");
    }
}