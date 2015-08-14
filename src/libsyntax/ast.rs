use std::collections::{HashMap};
//use visit::*;

struct ExpressionEvaluator;

/*impl SymbolVisitor for ExpressionEvaluator{
	pub fn visit_num(&self, num_exp : &NumExpression){
		
	}
	
	pub fn visit_ident(&self, num_exp : &IdentExpression){
		
	}
}*/

pub enum LuaType{
    LString(String),
    LNumber(i32),
    LFunction,
    LBool,
    LThread,
    LTable,
    LNil
}

pub trait Statement{
    fn generate_code(&self)->Vec<String>;
}

pub trait Expression{
    fn semantic(&self, &Block);
    //fn accept(&self, &SymbolVisitor);
}

pub struct Block{
    sym_tab : HashMap<String, LuaType>,
    pub statements : Vec<Box<Statement>>, //trait is boxed because it has no size known at compile-time. this is a trait object.
    pub instructions : Vec<String>
}

impl Block{
    pub fn new()->Self{
        Block {sym_tab : HashMap::new(), statements : Vec::new(), instructions : Vec::new()}
    }
    
    pub fn add_sym(&mut self, sym_id : String, value : LuaType){
        self.sym_tab.insert(sym_id, value);
    }
    
    pub fn contains(&self, sym_id : &String)->bool{
        match self.sym_tab.get(sym_id){
            Some(s) => true,
            _ => false
        }
    }
    
    
    pub fn generate(&mut self){
        for s in &self.statements{
            /*for i in &s.generate_code(){
                println!("{}", i);
            }*/
            self.instructions.extend(s.generate_code().into_iter());
        }
    }
}


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
	
	/*fn accept(&self, visitor: &SymbolVisitor){
		visitor.visit(self);
	}*/
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
//-----------------------------------------------------------------------------------------------------
pub struct AssignStatement{
    line_pos : usize,
    lhs_sym : IdentExpression,
    rhs_expr : Box<Expression>
}

impl AssignStatement{
    pub fn new(line_pos : usize, lhs_sym : IdentExpression, rhs_expr : Box<Expression>)->Self{
        AssignStatement {line_pos : line_pos, lhs_sym : lhs_sym, rhs_expr : rhs_expr}
    }
}

impl Statement for AssignStatement{
    fn generate_code(&self)->Vec<String>{
        vec!["MOV 1,2".to_string()]
    }
}

pub struct FuncCallStatement{
    line_pos : usize,
    name : String
}
impl FuncCallStatement{
    pub fn new(line_pos : usize, name : String) -> Self{
        FuncCallStatement {line_pos : line_pos, name : name}
    }
}

impl Statement for FuncCallStatement{
    fn generate_code(&self)->Vec<String>{
        vec!["MOV 1,2".to_string()]
    }
}

pub struct LabelStatement{
    line_pos : usize,
    name : String
}

impl LabelStatement{
    pub fn new(line_pos : usize, name : String) -> Self{
        LabelStatement {line_pos : line_pos, name : name}
    }
}

impl Statement for LabelStatement{
    fn generate_code(&self) -> Vec<String>{
        let mut s = self.name.clone();
        s.push(':');
        vec![s]
    }
}

pub struct BreakStatement{
    line_pos : usize
}

impl BreakStatement{
    pub fn new(line_pos : usize) -> Self{
        BreakStatement {line_pos : line_pos} 
    }
}

impl Statement for BreakStatement{
    fn generate_code(&self) -> Vec<String>{
        //TODO: should this be bra instead?
        vec!["jmp".to_string()]
    }
}

pub struct GotoStatement{
    line_pos : usize,
    label : String
}

impl GotoStatement{
    pub fn new(line_pos : usize, label : String) -> Self{
        GotoStatement {line_pos : line_pos, label : label}
    }
}

impl Statement for GotoStatement{
    fn generate_code(&self) -> Vec<String>{
        vec![format!("bra {}", self.label)]
    }
}

pub struct DoStatement{
    line_pos : usize,
    pub block : Block
}

impl DoStatement{
    pub fn new(line_pos : usize)->Self{
        DoStatement {line_pos : line_pos, block : Block::new()}
    }
}

impl Statement for DoStatement{
    fn generate_code(&self) -> Vec<String>{
        let mut instructions : Vec<String> = Vec::new(); 
        for s in &self.block.statements{
            instructions.extend(s.generate_code().into_iter());
        }
        instructions
    }
}

pub struct WhileStatement{
    line_pos : usize,
    expr : Box<Expression>,
    do_stat : DoStatement
}

impl WhileStatement{
    fn new(line_pos : usize, expr : Box<Expression>)->Self{
        WhileStatement {line_pos : line_pos, expr : expr, do_stat : DoStatement::new(line_pos)}
    }
}

impl Statement for WhileStatement{
    fn generate_code(&self) -> Vec<String>{
        let mut instructions : Vec<String> = Vec::new(); 
        for s in &self.do_stat.block.statements{
            instructions.extend(s.generate_code().into_iter());
        }
        instructions
    }
}

pub struct RepeatUntilStatement;