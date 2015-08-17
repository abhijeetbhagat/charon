#![allow(dead_code)]

use std::fmt;
use std::collections::{HashMap};
use visit::{Visitor};

struct ExpressionEvaluator;

/*impl SymbolVisitor for ExpressionEvaluator{
	pub fn visit_num(&self, num_exp : &NumExpression){
		
	}
	
	pub fn visit_ident(&self, num_exp : &IdentExpr){
		
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

impl Display for LuaType{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match *self{
            LuaType::LString
        }
    }
}

pub trait Statement{
    fn generate_code(&self)->Vec<String>;
}

pub struct Block{
    sym_tab : HashMap<String, LuaType>,
    pub statements : Vec<Box<Stmt>>, //trait is boxed because it has no size known at compile-time. this is a trait object.
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
            //self.instructions.extend(s.generate_code().into_iter());
        }
    }
}

pub enum Expr{
   NumExpr(i32),
   IdentExpr(String),
   AddExpr(Box<Expr>, Box<Expr>),
   SubExpr(Box<Expr>, Box<Expr>),
   MulExpr(Box<Expr>, Box<Expr>),
   DivExpr(Box<Expr>, Box<Expr>),
   ModExpr(Box<Expr>, Box<Expr>),
   BlockExpr(Box<Block>),
   IfElseExpr(Box<Expr>, Box<Block>, Box<Expr>),
   WhileExpr(Box<Expr>, Box<Block>),
   AssignExpr(String, Box<Expr>),
   LabelExpr(String),
   BreakExpr,
   GotoExpr(String)
   
}

pub struct Local{
    ident : String,
    ty : LuaType,
    expr : Box<Expr>
}

impl Local{
    pub fn new(ident : String, ty : LuaType, expr : Box<Expr>) -> Local{
        Local {ident : ident, ty : ty, expr : expr}
    }
    
}

pub enum Stmt{
     VarDeclStmt(Local),
     ExprStmt(Box<Expr>)
     //FnDecl()
}
/*pub struct SubExpression{
	e1 : Box<Expr>,
	e2 : Box<Expr>
}

impl SubExpression{
	pub fn new(e1 : Box<Expr>, e2 : Box<Expr>)->Self{
		SubExpression {e1 : e1, e2 : e2}
	}
}

impl Expr for SubExpression{
	fn semantic(&self, block: &Block){
		
	}
}

pub struct MulExpression{
	e1 : Box<Expr>,
	e2 : Box<Expr>
}

impl MulExpression{
	pub fn new(e1 : Box<Expr>, e2 : Box<Expr>)->Self{
		MulExpression {e1 : e1, e2 : e2}
	}
}

impl Expr for MulExpression{
	fn semantic(&self, block: &Block){
		
	}
}

pub struct DivExpression{
	e1 : Box<Expr>,
	e2 : Box<Expr>
}

impl DivExpression{
	pub fn new(e1 : Box<Expr>, e2 : Box<Expr>)->Self{
		DivExpression {e1 : e1, e2 : e2}
	}
}

impl Expr for DivExpression{
	fn semantic(&self, block: &Block){
		
	}
}

pub struct ModExpression{
	e1 : Box<Expr>,
	e2 : Box<Expr>
}

impl ModExpression{
	pub fn new(e1 : Box<Expr>, e2 : Box<Expr>)->Self{
		ModExpression {e1 : e1, e2 : e2}
	}
}

impl Expr for ModExpression{
	fn semantic(&self, block: &Block){
		
	}
}
*/
pub struct DotDotDotExpression;
//-----------------------------------------------------------------------------------------------------
pub struct AssignStatement{
    line_pos : usize,
    lhs_sym : Box<Expr>,
    rhs_expr : Box<Expr>
}

impl AssignStatement{
    pub fn new(line_pos : usize, lhs_sym : Box<Expr>, rhs_expr : Box<Expr>)->Self{
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
            //instructions.extend(s.generate_code().into_iter());
        }
        instructions
    }
}

pub struct WhileStatement{
    line_pos : usize,
    expr : Box<Expr>,
    do_stat : DoStatement
}

impl WhileStatement{
    fn new(line_pos : usize, expr : Box<Expr>)->Self{
        WhileStatement {line_pos : line_pos, expr : expr, do_stat : DoStatement::new(line_pos)}
    }
}

impl Statement for WhileStatement{
    fn generate_code(&self) -> Vec<String>{
        let mut instructions : Vec<String> = Vec::new(); 
        for s in &self.do_stat.block.statements{
            //instructions.extend(s.generate_code().into_iter());
        }
        instructions
    }
}

pub struct RepeatUntilStatement;