use ast::*;

pub trait SymbolVisitor{
	fn visit_num(&self, &NumExpression);
	fn visit_ident(&self, &IdentExpression);
}
