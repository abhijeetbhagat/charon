#![allow(dead_code)]

use std::collections::{HashMap};
use ast::*;
use visit::{Visitor};
use std::cell::RefCell;
use ptr::*;

pub type OptionalBinding = Option<B<Binding>>;

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
    fn visit_expr(&mut self, expr: &'a mut Expr){
        match *expr{
            //FIXME remove NilExpr; this is only for unit testing
            Expr::NilExpr => self.ty = TType::TString,
            Expr::NumExpr(_) => self.ty = TType::TInt32,
            Expr::StringExpr(_) => self.ty = TType::TString,
            Expr::IdExpr(ref mut id) =>{
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
            Expr::AddExpr(ref mut left, ref mut right) => {
                self.visit_expr(left);
                let left_ty = self.ty.clone();
                if left_ty != TType::TInt32{
                    panic!("Expected left operand of int type")
                }
                self.visit_expr(right);
                if self.ty != TType::TInt32{
                    panic!("Expected right operand of int type")
                }
            },
            Expr::DivExpr(ref mut left, ref mut  right) => {
                self.visit_expr(left);
                let left_ty = self.ty.clone();
                if left_ty != TType::TInt32{
                    panic!("Expected left operand of int type")
                }
                self.visit_expr(right);
                if self.ty != TType::TInt32{
                    panic!("Expected right operand of int type")
                }

                match **right{
                    Expr::NumExpr(n) => if n == 0 {panic!("Denominator cannot be 0")},
                    _ => {}
                }
            },
            Expr::SeqExpr(ref mut opt_expr_list) => {
                if opt_expr_list.is_none(){
                    self.ty = TType::TVoid;
                    return
                }
                for b_expr in opt_expr_list.as_mut().unwrap() {
                    self.visit_expr(&mut *b_expr);
                }
            },
            Expr::IfThenElseExpr(ref mut conditional_expr, ref mut then_expr, ref mut else_expr) => {
                self.visit_expr(conditional_expr);
                if self.ty != TType::TInt32{
                    panic!("Expected conditional expression of int type");
                }
                self.visit_expr(then_expr);
                let then_ty = self.ty.clone();
                self.visit_expr(else_expr);
                if then_ty != self.ty{
                    panic!("Expected then expr and else expr types to be same");
                }
            },
            Expr::IfThenExpr(ref mut conditional_expr, ref mut then_expr) => {
                self.visit_expr(conditional_expr);
                if self.ty != TType::TInt32{
                    panic!("Expected conditional expression of int type");
                }
                self.visit_expr(then_expr);
                if self.ty != TType::TVoid{
                    panic!("Expected if-body of void type")
                }
            },
            Expr::WhileExpr(ref mut conditional_expr, ref mut body) => {
                self.visit_expr(conditional_expr);
                if self.ty != TType::TInt32{
                    panic!("Expected conditional expression of int type");
                }

                self.visit_expr(body);
                if self.ty != TType::TVoid{
                    panic!("Expected while-body of void type")
                }
            },
            Expr::ForExpr(ref mut id, ref mut from, ref mut to, ref mut body) => {
                self.visit_expr(from);
                if self.ty != TType::TInt32{
                    panic!("Initializing expression type should be int in a for loop");
                }

                self.visit_expr(to);
                if self.ty != TType::TInt32{
                    panic!("To expression type should be int in a for loop");
                }
            },
            Expr::CallExpr(ref id, ref mut optional_ty_expr_list) => {
                //fix call expr return type by doing a sym-tab lookup
                for &mut (ref mut ty, ref mut expr) in optional_ty_expr_list.as_mut().unwrap(){
                    match **expr{
                        Expr::CallExpr(ref id, _) => {
                            for &(ref _id, ref binding) in self.sym_tab.iter().rev(){
                                if *id == *_id{
                                    match **binding.as_ref().unwrap(){
                                        Binding::FuncBinding(ref _ty) => {
                                            *ty = _ty.clone();
                                            break;
                                        } ,
                                        _ => {}
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            },
            Expr::LetExpr(ref mut decls, ref mut opt_expr) => {
                self.sym_tab.push(("<marker>".to_string(), None));

                for dec in decls{ //decls is a &
                    self.visit_decl(dec);
                }

                match *opt_expr {
                    Some(ref mut b_expr) => {
                        self.visit_expr(&mut *b_expr);
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

    fn visit_decl(&mut self, decl : &'a mut Decl){
        macro_rules! store_into_sym_tab {
            ($self_ : ident, $i : ident, $p : path) => {
                $self_.sym_tab.push(($i.clone(), Some(B($p($self_.ty.clone())))));
            }
        }
        match *decl{
            Decl::VarDec(ref id, ref ty, ref mut expr) => {
                self.visit_expr(expr);
                if *ty != self.ty{
                    panic!("Types mismatch")
                }
                store_into_sym_tab!(self, id, Binding::VarBinding);
            },
            Decl::FunDec(ref id, ref params, ref ret_type, ref body, ref body_type) => {
                if ret_type != body_type {
                    panic!("Return type doesn't match with the type of the last expression.")
                }

                if params.is_some() {
                    let mut map = HashMap::new();
                    for p in params.as_ref().unwrap(){
                        if map.contains_key(&p.0){
                            panic!(format!("Duplicate param '{0}' found", p.0));
                        }
                        map.insert(&p.0, true);
                    }
                }
                //self.visit_expr(&body);
                //if self.ty != *ret_type{
                println!("pushing {0}", id);
                self.ty = ret_type.clone();
                store_into_sym_tab!(self, id, Binding::FuncBinding);
            },
            Decl::TypeDec(ref id, ref ty) => {
                store_into_sym_tab!(self, id, Binding::TypeBinding);
            }
        }
    }
}

#[test]
fn test_ty_set_for_num() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::NumExpr(23));
    assert_eq!(TType::TInt32, v.ty);
}

#[test]
fn test_ty_set_for_int_id() {
    let mut v = TypeChecker::new();
    v.sym_tab.push(("a".to_string(), Some(B(Binding::VarBinding(TType::TInt32)))));
    v.visit_expr(&mut Expr::IdExpr("a".to_string()));
    assert_eq!(TType::TInt32, v.ty);
}

#[test]
fn test_type_match_int_for_var_dec() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut Decl::VarDec("a".to_string(), TType::TInt32, B(Expr::NumExpr(4))));
    assert_eq!(TType::TInt32, v.ty);
    assert_eq!(v.sym_tab.len(), 1);
    assert_eq!(v.sym_tab[0].0, "a".to_string());
    //assert_eq!(v.sym_tab[0].1, TType::TInt32);
}

#[test]
fn test_type_match_string_for_var_dec() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut Decl::VarDec("a".to_string(), TType::TString, B(Expr::NilExpr)));
    assert_eq!(TType::TString, v.ty);
    assert_eq!(v.sym_tab.len(), 1);
    assert_eq!(v.sym_tab[0].0, "a".to_string());
    //assert_eq!(v.sym_tab[0].1, TType::TString);
}

#[test]
#[should_panic]
fn test_type_check_for_var_dec_type_mismatch() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut Decl::VarDec("a".to_string(), TType::TInt32, B(Expr::NilExpr)));
    assert_eq!(TType::TInt32, v.ty);
}

#[test]
fn test_correct_types_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::AddExpr(B(Expr::NumExpr(4)), B(Expr::NumExpr(4))));
    assert_eq!(v.ty, TType::TInt32);
}

#[test]
#[should_panic(expected="Expected left operand of int type")]
fn test_left_type_invalid_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::AddExpr(B(Expr::NilExpr), B(Expr::NumExpr(4))));
}

#[test]
#[should_panic(expected="Expected right operand of int type")]
fn test_right_type_invalid_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::AddExpr(B(Expr::NumExpr(4)), B(Expr::NilExpr)));
}

#[test]
fn test_var_hiding() {
    //let mut v =
}

#[test]
fn test_func_decl_correct_return_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut Decl::FunDec(String::from("foo"), None, TType::TInt32, B(Expr::NumExpr(4)), TType::TInt32));
}

#[test]
#[should_panic(expected="Return type doesn't match with the type of the last expression.")]
fn test_func_decl_incorrect_return_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut Decl::FunDec(String::from("foo"), None, TType::TString, B(Expr::NumExpr(4)), TType::TInt32));
}

#[test]
#[should_panic(expected="Expected conditional expression of int type")]
fn test_if_expr_with_incorrect_conditional_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::IfThenExpr(B(Expr::StringExpr(String::from("a"))), B(Expr::StringExpr(String::from("a")))));
}

#[test]
fn test_if_expr_with_int_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::IfThenExpr(B(Expr::NumExpr(1)), B(Expr::SeqExpr(None))));
    assert_eq!(v.ty, TType::TVoid);
}

#[test]
#[should_panic(expected="Expected if-body of void type")]
fn test_if_expr_with_int_type_conditional_and_int_type_as_body_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::IfThenExpr(B(Expr::NumExpr(1)), B(Expr::NumExpr(1))));
    assert_eq!(v.ty, TType::TInt32);
}

#[test]
fn test_if_else_expr_with_matching_types() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::IfThenElseExpr(B(Expr::NumExpr(1)), B(Expr::NumExpr(1)), B(Expr::NumExpr(1))));
    assert_eq!(v.ty, TType::TInt32);
}

#[test]
#[should_panic(expected="Expected then expr and else expr types to be same")]
fn test_if_else_expr_with_non_matching_types() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::IfThenElseExpr(B(Expr::NumExpr(1)), B(Expr::NumExpr(1)), B(Expr::StringExpr(String::from("a")))));
}

#[test]
#[should_panic(expected="Expected conditional expression of int type")]
fn test_while_expr_with_incorrect_conditional_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::WhileExpr(B(Expr::StringExpr(String::from("a"))), B(Expr::StringExpr(String::from("a")))));
}

#[test]
#[should_panic(expected="Expected while-body of void type")]
fn test_while_expr_with_int_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::WhileExpr(B(Expr::NumExpr(1)), B(Expr::StringExpr(String::from("a")))));
}

#[test]
#[should_panic(expected="Denominator cannot be 0")]
fn test_div_expr_with_0_as_denominator(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::DivExpr(B(Expr::NumExpr(1)), B(Expr::NumExpr(0))));
}

#[test]
fn test_div_expr_with_1_as_denominator(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::DivExpr(B(Expr::NumExpr(1)), B(Expr::NumExpr(1))));
    assert_eq!(v.ty, TType::TInt32);
}

#[test]
#[should_panic(expected="Initializing expression type should be int in a for loop")]
fn test_for_loop_expr_init_type(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut Expr::ForExpr(String::from("i"),
                                B(Expr::StringExpr(String::from("adsd"))),
                                B(Expr::NumExpr(1)),
                                B(Expr::NumExpr(2))));
}

#[test]
#[should_panic(expected="Duplicate param 'a' found")]
fn test_func_dec_with_duplicate_param_with_same_type(){
    let mut v = TypeChecker::new();
    v.visit_decl(&mut Decl::FunDec(String::from("foo"), 
                               Some(vec![(String::from("a"), TType::TInt32),
                                         (String::from("a"), TType::TInt32) ]),
                               TType::TInt32,
                               B(Expr::NumExpr(4)),
                               TType::TInt32));
}

#[test]
#[should_panic(expected="Duplicate param 'a' found")]
fn test_func_dec_with_duplicate_param_with_different_types(){
    let mut v = TypeChecker::new();
    v.visit_decl(&mut Decl::FunDec(String::from("foo"), 
                               Some(vec![(String::from("a"), TType::TInt32),
                                         (String::from("a"), TType::TString) ]),
                               TType::TInt32,
                               B(Expr::NumExpr(4)),
                               TType::TInt32));
}

#[test]
fn test_call_expr_ret_type_fix(){
    let mut v = TypeChecker::new();
    let e = &mut Expr::LetExpr(vec![Decl::FunDec(String::from("foo"), 
                               Some(vec![(String::from("a"), TType::TInt32)]),
                               TType::TInt32,
                               B(Expr::NumExpr(4)),
                               TType::TInt32)],
                               Some(B(Expr::CallExpr(String::from("foo"),
                                                     Some(vec![(TType::TVoid, B(Expr::CallExpr(String::from("foo"),
                                                                                               Some(vec![(TType::TInt32, B(Expr::NumExpr(2)))])
                                                                                              )
                                                                               )
                                                               )]
                                                         )
                                                    )
                                     )
                                   )
                               );

    v.visit_expr(e);
    match *e{
        Expr::LetExpr(_, ref e) => {
            match **e.as_ref().unwrap(){
                Expr::CallExpr(_, ref l) => {
                    let ul = l.as_ref().unwrap();
                    match ul[0]{
                        (TType::TInt32, _) => {},
                        ref t => {
                            println!("{:?}", t);
                            panic!("failed");
                        }
                    }
                },
                _ => panic!("failed2")
            }
        },
        _ => panic!("failed3")
    }
}
