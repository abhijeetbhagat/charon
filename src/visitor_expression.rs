use expressions::*;

pub trait SymbolVisitor{
	fn visit_num(&self, &NumExpression);
	fn visit_ident(&self, &IdentExpression);
}

struct ExpressionEvaluator;

impl SymbolVisitor for ExpressionEvaluator{
	pub fn visit_num(&self, num_exp : &NumExpression){
		
	}
	
	pub fn visit_ident(&self, num_exp : &IdentExpression){
		
	}
}