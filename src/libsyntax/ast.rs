#![allow(dead_code)]

use std::fmt;
use std::collections::{HashMap, BTreeMap};
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
pub type OptionalExprList = Option<Vec<B<Expr>>>;
pub type OptionalExpr = Option<B<Expr>>;
pub type OptionalTypeExprTupleList = Option<Vec<(TType, B<Expr>)>>;
pub type OptionalParamInfoList = Option<Vec<(String, TType)>>;

#[derive(Debug, PartialEq, Clone)]
pub enum TType{
    TInt32,
    TString,
    TArray(B<TType>), //TType can be anything
    TRecord,
    TCustom(String),
    TNil,
    TVoid
}

pub enum TypeValue{
    TInt32(i32),
    TString(String),
    TIdent(String)
}

impl fmt::Display for TType{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match *self{
            TType::TInt32 => f.write_str("Number"),
            TType::TString => f.write_str("String"),
            TType::TArray(ref T) => f.write_str("Array of some type"),
            TType::TRecord => f.write_str("Record"),
            TType::TCustom(ref name) => f.write_str("Custom"),
            TType::TNil => f.write_str("Nil"),
            TType::TVoid => f.write_str("Void")
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
   //let dec+ in exp*; end
   //note: instead of making a list of exprs as the grammar suggests,
   //use a seq-expr. This will make parsing easier.
   //i.e. we don't want this:
   //let var v := 6
   //in print(v);
   //   print("a");
   //   print("b")
   //end
   //instead we want this (note the parens):
   //let var v:= 6
   //in (print(v);
   //    print("a");
   //    print("b")
   //   )
   //end
   LetExpr(Vec<Decl>, OptionalExpr),
   //id
   IdExpr(String),
   //nil
   NilExpr,
   //FIXME is this needed?
   LitExpr,
   //stringLit
   StringExpr(String),
   //break
   BreakExpr,
   //id ( exp*, )
   CallExpr(String, OptionalTypeExprTupleList),
   //intLit
   NumExpr(i32),
   //( exp*; )
   SeqExpr(OptionalExprList),

   IdentExpr(String),
   AddExpr(B<Expr>, B<Expr>),
   SubExpr(B<Expr>, B<Expr>),
   MulExpr(B<Expr>, B<Expr>),
   DivExpr(B<Expr>, B<Expr>),
   ModExpr(B<Expr>, B<Expr>),
   BlockExpr(B<Block>),
   IfElseExpr(B<Expr>, B<Expr>, B<Expr>),
   WhileExpr(B<Expr>, B<Expr>),
   AssignExpr(String, B<Expr>),
   LabelExpr(String),
   GotoExpr(String)
}

pub struct FieldDec{
    id : String,
    ty : TType
}

//lst.where(move |x|{x.id == "id"}).first()

pub enum Decl{
    //type tyId = ty
    TyDec(String, TType),
    //var a : int := 1
    VarDec(String, TType, B<Expr>),
    //function id ( fieldDec; ) : tyId = exp
    FunDec(String, OptionalParamInfoList, TType, B<Expr>)
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
}
