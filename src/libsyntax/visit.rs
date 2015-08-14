

/*pub trait SymbolVisitor{
	fn visit_num(&self, &NumExpression);
	fn visit_ident(&self, &IdentExpression);
}*/

pub trait Visitor<T> {
    fn visit(&self, t: T);
}

//struct CodeGenVisitor;
/*fn main() {
    let visitor = PrintVisitor;
    
    visitor.visit(1);
    visitor.visit("Hello, world");
    visitor.visit(NotDisplayable);
}*/