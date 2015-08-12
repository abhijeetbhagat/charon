use trait_expression::*;
use block::*;

pub struct NumExpression{
	value : i32
}

impl NumExpression{
	pub fn new(value : i32)->Self{
		NumExpression {value : value}
	}
}

impl Expression for NumExpression{
	fn semantic(&self, block: &Block){}
}

pub struct IdentExpression{
	value : String
}

impl IdentExpression{
	pub fn new(value : String)->Self{
		IdentExpression {value : value}
	}
}

impl Expression for IdentExpression{
	fn semantic(&self, block: &Block){}
}

pub struct DotDotDotExpression;