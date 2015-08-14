use trait_expression::*;
use visitor_expression::*;
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
	fn semantic(&self, block: &Block){
		let reg = "rsp";
		let offset = "1";
		println!("mov dword ptr[{}+{}], {}", reg, offset, format!("{:X}", self.value));
	}
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
	
	fn accept(&self, visitor: &SymbolVisitor){
		visitor.visit(self);
	}
}

pub struct AddExpression{
	e1 : Box<Expression>,
	e2 : Box<Expression>
}

impl AddExpression{
	pub fn new(e1 : Box<Expression>, e2 : Box<Expression>)->Self{
		AddExpression {e1 : e1, e2 : e2}
	}
}

impl Expression for AddExpression{
	fn semantic(&self, block: &Block){
		
	}
}

pub struct SubExpression{
	e1 : Box<Expression>,
	e2 : Box<Expression>
}

impl SubExpression{
	pub fn new(e1 : Box<Expression>, e2 : Box<Expression>)->Self{
		SubExpression {e1 : e1, e2 : e2}
	}
}

impl Expression for SubExpression{
	fn semantic(&self, block: &Block){
		
	}
}

pub struct MulExpression{
	e1 : Box<Expression>,
	e2 : Box<Expression>
}

impl MulExpression{
	pub fn new(e1 : Box<Expression>, e2 : Box<Expression>)->Self{
		MulExpression {e1 : e1, e2 : e2}
	}
}

impl Expression for MulExpression{
	fn semantic(&self, block: &Block){
		
	}
}

pub struct DivExpression{
	e1 : Box<Expression>,
	e2 : Box<Expression>
}

impl DivExpression{
	pub fn new(e1 : Box<Expression>, e2 : Box<Expression>)->Self{
		DivExpression {e1 : e1, e2 : e2}
	}
}

impl Expression for DivExpression{
	fn semantic(&self, block: &Block){
		
	}
}

pub struct ModExpression{
	e1 : Box<Expression>,
	e2 : Box<Expression>
}

impl ModExpression{
	pub fn new(e1 : Box<Expression>, e2 : Box<Expression>)->Self{
		ModExpression {e1 : e1, e2 : e2}
	}
}

impl Expression for ModExpression{
	fn semantic(&self, block: &Block){
		
	}
}

pub struct DotDotDotExpression;