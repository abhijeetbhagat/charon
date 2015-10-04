#![allow(dead_code)]

use ast::*;
use visit::{Visitor};
use std::cell::RefCell;
use ptr::*;


pub type OptionalBinding = Option<B<Binding>>;
/*impl<T> Visitor<T> for CodeGenVisitor where T: std::fmt::Display {
    fn visit(&self, t: T) {
        println!("{}", t);
    }
}*/

/*struct CodeGenVisitor;
struct ExpVisitor{
    result : i32
}

impl<'a> Visitor<&'a NumExpression> for ExpVisitor {
    fn visit(&mut self, n: &'a NumExpression) {
        self.result = n.value;
    }
}

impl<'a> Visitor<&'a AddExpression> for ExpVisitor {
    fn visit(&mut self, add: &'a AddExpression) {
        let b1 = *(add.e1);
        b1.accept(self);
        let r1 = self.result;
        add.e2.accept(self);
        let r2 = self.result;
        self.result = r1 + r2;
    }
}*/

struct ExpEvaluator{
    value : i32
}

impl ExpEvaluator{
    fn get_value(&mut self, expr : & Expr)->i32{
        match expr{
            &Expr::NumExpr(value) => value,
            _ => 1
        }
    }
}

impl<'a> Visitor<'a> for ExpEvaluator{
    fn visit_expr(&mut self, expr: &'a Expr){
        match expr{
            &Expr::AddExpr(ref left, ref right) => {
                let v1 = self.get_value(left);
                let v2 = self.get_value(right);
                self.value = v1 + v2;
            },
            _ => {}
        }
    }
}

pub struct TypeChecker{
    //block_stack : Vec<RefCell<&'a  Block>>,
    pub sym_tab : Vec<(String, OptionalBinding)>,
    //decl_cnt : u32,
    //decl_cnt_stack : Vec<u32>,
    pub ty : TType
}

impl TypeChecker{
    pub fn new()->Self{
        TypeChecker {sym_tab : Vec::new(), ty : TType::TNil}
    }

    fn get_type_for(&self){//}->&TType{
        //self.block_stack
    }
}

impl<'a> Visitor<'a> for TypeChecker{
    fn visit_expr(&mut self, expr: &'a Expr){
        match expr{
            //FIXME remove NilExpr; this is only for unit testing
            &Expr::NilExpr => self.ty = TType::TString,
            &Expr::NumExpr(_) => self.ty = TType::TInt32,
            &Expr::StringExpr(_) => self.ty = TType::TString,
            &Expr::IdentExpr(ref id) =>{
                //search in the symtab for id's existence and get the type
                let mut found = false;
                for &(ref _id, ref _binding) in &self.sym_tab{ //iterator returns a ref to tuple while iterating; so &(_,_) has to be used
                    if *_id == *id{
                        found = true;
                        self.ty = match **_binding.as_ref().unwrap(){
                            Binding::TypeBinding(ref ty) |
                            Binding::VarBinding(ref ty) |
                            Binding::FuncBinding(ref ty) => ty.clone()
                        };
                        break;
                    }
                }
                if !found{
                    panic!("{} not found", id);
                }
            },
            &Expr::AddExpr(ref left, ref right) => {
                self.visit_expr(left);
                let left_ty = self.ty.clone();
                if left_ty != TType::TInt32{
                    panic!("Expected left operand of int type");
                }
                self.visit_expr(right);
                if self.ty != TType::TInt32{
                    panic!("Expected right operand of int type")
                }
            },
            &Expr::SeqExpr(ref opt_expr_list) => {
                for b_expr in opt_expr_list.as_ref().unwrap() {
                    self.visit_expr(&*b_expr);
                }
            },
            &Expr::LetExpr(ref decls, ref opt_expr) => {
                self.sym_tab.push(("<marker>".to_string(), None));

                for dec in decls{ //decls is a &
                    self.visit_decl(dec);
                }

                match opt_expr {
                    &Some(ref b_expr) => {
                        self.visit_expr(&*expr);
                    },
                    _ => {}
                }
                //pop till marker and then pop marker
                while self.sym_tab.last().unwrap().0 != "<marker>".to_string(){
                    self.sym_tab.pop();
                }
                self.sym_tab.pop();
            },

            _ => {}
        }
    }

    fn visit_decl(&mut self, decl : &'a Decl){
        match decl{
            &Decl::VarDec(ref id, ref ty, ref expr) => {
                self.visit_expr(expr);
                if *ty != self.ty{
                    panic!("Types mismatch");
                }
                self.sym_tab.push((id.clone(), Some(B(Binding::VarBinding(self.ty.clone())))));
            },
            &Decl::FunDec(ref id, ref params, ref ret_type, ref body) => {
                self.visit_expr(&body);
                if self.ty != *ret_type{
                    panic!("Return type doesn't match with the type of the last expression.");
                }
                self.sym_tab.push((id.clone(), Some(B(Binding::FuncBinding(ret_type.clone())))));
            },
            &Decl::TypeDec(ref id, ref ty) => {
                self.sym_tab.push((id.clone(), Some(B(Binding::TypeBinding(self.ty.clone())))));
            }
        }
    }
}

struct PrettyPrintVisitor;

impl<'a> Visitor<'a> for PrettyPrintVisitor{
    fn visit_expr(&mut self, expr:&'a Expr){
        match expr{
            &Expr::LetExpr(ref v, _) => {
                println!("(let");
                for d in v{
                    println!("\t(");

                }
                println!(")");
            },
            &Expr::AddExpr(ref left, ref right) => {
                self.visit_expr(left);
                println!(" Plus ");
                self.visit_expr(right);
            },
            &Expr::NumExpr(value) => {
                println!(" Num({}) ", value);
            },
            &Expr::IdentExpr(ref value) => {
                println!(" Ident({}) ", value);
            },
            _ => {}
        }
    }

    fn visit_block(&mut self, block: &'a Block){
        for s in &block.statements{
            self.visit_stmt(&*s);
        }
    }

    fn visit_stmt(&mut self, stmt : &'a Stmt){
        match stmt{
            &Stmt::VarDeclStmt(ref local) => {
                println!("(var {} type {} init ", local.ident, local.ty);
                self.visit_expr(&*local.expr);
                println!(")");
            },
            &Stmt::ExprStmt(ref expr) => {
                self.visit_expr(expr);
            }
        }
    }
}

//FIXME remove this struct
struct SymbolTableBuilder<'a>{
    block_stack : Vec<RefCell<&'a  Block>>
}

impl<'a> SymbolTableBuilder<'a>{
    fn new()->Self{
        SymbolTableBuilder {block_stack : Vec::new()}
    }
}

impl<'a> Visitor<'a> for SymbolTableBuilder<'a>{
    fn visit_block(&mut self, block : &'a Block){
        self.block_stack.push(RefCell::new(block));
        for s in &block.statements{
            self.visit_stmt(s);
        }
        self.block_stack.pop();
    }

    fn visit_stmt(&mut self, stmt : &'a Stmt){
        match stmt{
            &Stmt::VarDeclStmt(ref local) => {
                    //FIXME deduce the correct type
                    let mut block = self.block_stack.last_mut().unwrap().borrow_mut();
                    block.sym_tab.borrow_mut().insert(local.ident.clone(), TType::TInt32);
                },
            &Stmt::ExprStmt(ref expr) => {
                let b =  &**expr; //*expr is deref B which is Box<T>; **expr is deref Box<T> which is T; &**expr is therefore &T
                match b {
                    &Expr::BlockExpr(ref block) => {
                        self.visit_block(block);
                    },
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn test_pp_visit_add_expr(){
    let mut p = PrettyPrintVisitor;
    p.visit_expr(&Expr::AddExpr(B(Expr::NumExpr(1)), B(Expr::NumExpr(2))));
}

#[test]
fn test_pp_visit_block(){
    let mut p = PrettyPrintVisitor;
    let mut b = Block::new();
    // let l = Local::new("a".to_string(), LuaType::LNil, B(Expr::NumExpr(1)));
    // b.statements.push(B(Stmt::VarDeclStmt(l)));
    p.visit_block(&b);
}

//#[test]
fn test_pp_visit_add(){
    let mut p = PrettyPrintVisitor;
    let mut b = Block::new();
    p.visit_block(&b);
}

#[test]
fn test_ty_set_for_num() {
    let mut v = TypeChecker::new();
    v.visit_expr(&Expr::NumExpr(23));
    assert_eq!(TType::TInt32, v.ty);
}

#[test]
fn test_ty_set_for_int_id() {
    let mut v = TypeChecker::new();
    v.sym_tab.push(("a".to_string(), Some(B(Binding::VarBinding(TType::TInt32)))));
    v.visit_expr(&Expr::IdentExpr("a".to_string()));
    assert_eq!(TType::TInt32, v.ty);
}

#[test]
fn test_type_match_int_for_var_dec() {
    let mut v = TypeChecker::new();
    v.visit_decl(&Decl::VarDec("a".to_string(), TType::TInt32, B(Expr::NumExpr(4))));
    assert_eq!(TType::TInt32, v.ty);
    assert_eq!(v.sym_tab.len(), 1);
    assert_eq!(v.sym_tab[0].0, "a".to_string());
    //assert_eq!(v.sym_tab[0].1, TType::TInt32);
}

#[test]
fn test_type_match_string_for_var_dec() {
    let mut v = TypeChecker::new();
    v.visit_decl(&Decl::VarDec("a".to_string(), TType::TString, B(Expr::NilExpr)));
    assert_eq!(TType::TString, v.ty);
    assert_eq!(v.sym_tab.len(), 1);
    assert_eq!(v.sym_tab[0].0, "a".to_string());
    //assert_eq!(v.sym_tab[0].1, TType::TString);
}

#[test]
#[should_panic]
fn test_type_check_for_var_dec_type_mismatch() {
    let mut v = TypeChecker::new();
    v.visit_decl(&Decl::VarDec("a".to_string(), TType::TInt32, B(Expr::NilExpr)));
    assert_eq!(TType::TInt32, v.ty);
}

#[test]
fn test_correct_types_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&Expr::AddExpr(B(Expr::NumExpr(4)), B(Expr::NumExpr(4))));
    assert_eq!(v.ty, TType::TInt32);
}

#[test]
#[should_panic(expected="Expected left operand of int type")]
fn test_left_type_invalid_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&Expr::AddExpr(B(Expr::NilExpr), B(Expr::NumExpr(4))));
}

#[test]
#[should_panic(expected="Expected right operand of int type")]
fn test_right_type_invalid_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&Expr::AddExpr(B(Expr::NumExpr(4)), B(Expr::NilExpr)));
}

#[test]
fn test_var_hiding() {
    //let mut v =
}

#[test]
fn test_func_decl_correct_return_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&Decl::FunDec(String::from("foo"), None, TType::TInt32, B(Expr::NumExpr(4))));
}

#[test]
#[should_panic(expected="Return type doesn't match with the type of the last expression.")]
fn test_func_decl_incorrect_return_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&Decl::FunDec(String::from("foo"), None, TType::TString, B(Expr::NumExpr(4))));
}
