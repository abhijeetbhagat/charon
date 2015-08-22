#![allow(dead_code)]

use ast::*;
use visit::{Visitor};
use std::cell::RefCell;
use ptr::*;

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

struct TypeChecker{
    ty : TType
}

impl<'a> Visitor<'a> for TypeChecker{
    fn visit_expr(&mut self, expr: &'a Expr){
        match expr{
            &Expr::AddExpr(ref left, ref right) => {

            },
            _ => {}
        }
    }
}

struct PrettyPrintVisitor;

impl<'a> Visitor<'a> for PrettyPrintVisitor{
    fn visit_expr(&mut self, expr:&'a Expr){
        match expr{
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

#[test]
fn test_pp_visit_add(){
    let mut p = PrettyPrintVisitor;
    let mut b = Block::new();
    // let l = Local::new("a".to_string(), LuaType::LNil,
    //                     B(Expr::AddExpr(
    //                                     B(Expr::NumExpr(1)),
    //                                     B(Expr::NumExpr(2)))));
    // b.statements.push(B(Stmt::VarDeclStmt(l)));
    p.visit_block(&b);
}

// #[test]
// fn test_st_visit_block_two_same_var_decls(){
//     let mut b = Block::new();
//     let mut stb = SymbolTableBuilder::new();
//     // let l = Local::new("a".to_string(), LuaType::LNil, B(Expr::NumExpr(1)));
//     // b.statements.push(B(Stmt::VarDeclStmt(l)));
//     // let l2 = Local::new("a".to_string(), LuaType::LNil, B(Expr::NumExpr(1)));
//     // b.statements.push(B(Stmt::VarDeclStmt(l2)));
//     stb.visit_block(&b);
//     assert_eq!(b.sym_tab.borrow().len(), 1);
// }
//
//
// #[test]
// fn test_st_visit_block_two_diff_var_decls(){
//     let mut b = Block::new();
//     let mut stb = SymbolTableBuilder::new();
//     // let l = Local::new("a".to_string(), LuaType::LNil, B(Expr::NumExpr(1)));
//     // b.statements.push(B(Stmt::VarDeclStmt(l)));
//     // let l2 = Local::new("b".to_string(), LuaType::LNil, B(Expr::NumExpr(1)));
//     // b.statements.push(B(Stmt::VarDeclStmt(l2)));
//     stb.visit_block(&b);
//     assert_eq!(b.sym_tab.borrow().len(), 2);
// }
