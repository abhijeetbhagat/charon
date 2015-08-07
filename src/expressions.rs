pub struct NumExpression{
	value : i32
}

impl NumExpression{
	pub fn new(value : i32)->Self{
		NumExpression {value : value}
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