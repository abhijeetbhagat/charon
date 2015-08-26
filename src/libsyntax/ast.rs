#![allow(dead_code)]

use std::fmt;
use std::collections::{HashMap};
use visit::{Visitor};
use ptr::{B};
use std::cell::RefCell;

struct ExpressionEvaluator;

/*
impl SymbolVisitor for ExpressionEvaluator{
	pub fn visit_num(&self, num_exp : &NumExpression){

	}

	pub fn visit_ident(&self, num_exp : &IdentExpr){

	}
}
*/
#[derive(Debug, PartialEq, Clone)]
pub enum TType{
    TInt32,
    TString,
    TArray(B<TType>), //TType can be anything
    TRecord,
    TCustom(String),
    TNil,
}

impl fmt::Display for TType{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match *self{
            TType::TInt32 => f.write_str("Number"),
            TType::TString => f.write_str("String"),
            TType::TArray(ref T) => f.write_str("Array of some type"),
            TType::TRecord => f.write_str("Record"),
            TType::TCustom(ref name) => f.write_str("Custom"),
            TType::TNil => f.write_str("Nil")
        }
    }
}

pub trait Statement{
    fn generate_code(&self)->Vec<String>;
}

pub struct Block{
    pub sym_tab : RefCell<HashMap<String, TType>>,
    pub statements : Vec<B<Stmt>>, //trait is boxed because it has no size known at compile-time. this is a trait object.
    pub instructions : Vec<String>,
    pub expr : Option<B<Expr>>, //this holds the main expr as in the production program -> expr
    // pub child_block : Option<Block>,
    // pub parent_block : Option<Block>
}

impl Block{
    pub fn new()->Self{
        Block {sym_tab : RefCell::new(HashMap::new()),
               statements : Vec::new(),
               instructions : Vec::new(),
               expr : None,
            //    child_block : None,
            //    parent_block : None

        }
    }

    pub fn add_sym(&mut self, sym_id : String, ty : TType){
        self.sym_tab.borrow_mut().insert(sym_id, ty);
    }

    pub fn contains(&self, sym_id : &String)->bool{
        //FIXME use contains key
        match self.sym_tab.borrow().get(sym_id){
            Some(s) => true,
            _ => false
        }
        //false
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
   LetExpr(Vec<Decl>, Option<Vec<B<Expr>>>),
   IdExpr(String),
   NilExpr,
   LitExpr,
   StringExpr,
   BreakExpr,
   CallExpr(String, Option<B<Expr>>),
   NumExpr(i32),


   IdentExpr(String),
   AddExpr(B<Expr>, B<Expr>),
   SubExpr(B<Expr>, B<Expr>),
   MulExpr(B<Expr>, B<Expr>),
   DivExpr(B<Expr>, B<Expr>),
   ModExpr(B<Expr>, B<Expr>),
   BlockExpr(B<Block>),
   IfElseExpr(B<Expr>, B<Block>, B<Expr>),
   WhileExpr(B<Expr>, B<Block>),
   AssignExpr(String, B<Expr>),
   LabelExpr(String),
   GotoExpr(String)
}

pub struct FieldDec{
    id : String,
    ty : TType
}

pub enum Decl{
    TyDec(String, TType),
    VarDec(String, TType, B<Expr>),
    FunDec(String, Option<Vec<FieldDec>>)
}

pub struct Local{
    pub ident : String,
    pub ty : TType,
    pub expr : B<Expr>
}

impl Local{
    pub fn new(ident : String, ty : TType, expr : B<Expr>) -> Local{
        Local {ident : ident, ty : ty, expr : expr}
    }
}

pub enum Stmt{
     VarDeclStmt(Local),
     ExprStmt(B<Expr>)
     //FnDecl()
}

/*
pub struct SubExpression{
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
