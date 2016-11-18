#![allow(dead_code)]

use itertools::Itertools;
use std::collections::{HashMap};
use ast::{Binding, Expr, Decl, TType, OptionalIdTypePairs};
use ast::Binding::*;
use ast::Expr::*;
use ast::TType::*;
use ast::Decl::*;
use visit::{Visitor};
use std::cell::RefCell;
use ptr::*;

pub type OptionalBinding = Option<B<Binding>>;

pub struct TypeChecker{
    //block_stack : Vec<RefCell<&'a  Block>>,
    pub sym_tab : Vec<(String, OptionalBinding)>,
    std_functions : HashMap<String, Binding>,
    //decl_cnt : u32,
    //decl_cnt_stack : Vec<u32>,
    pub ty : TType 
}

impl TypeChecker{
    pub fn new()->Self{
        let mut std_functions = HashMap::new();
        std_functions.insert(String::from("print"), FuncBinding(TVoid));
        std_functions.insert(String::from("flush"), FuncBinding(TVoid));
        std_functions.insert(String::from("getchar"), FuncBinding(TString));
        std_functions.insert(String::from("ord"), FuncBinding(TInt32));
        std_functions.insert(String::from("chr"), FuncBinding(TString));
        std_functions.insert(String::from("size"), FuncBinding(TInt32));
        std_functions.insert(String::from("substring"), FuncBinding(TString));
        std_functions.insert(String::from("concat"), FuncBinding(TString));
        std_functions.insert(String::from("not"), FuncBinding(TInt32));
        std_functions.insert(String::from("exit"), FuncBinding(TVoid));

        TypeChecker {
            sym_tab : Vec::new(),
            ty : TNil,
            std_functions : std_functions,
        }
    }

    fn get_type_for(&self){//}->&TType{
        //self.block_stack
    }
}

impl<'a> Visitor<'a> for TypeChecker{
    fn visit_expr(&mut self, expr: &'a mut Expr){
        macro_rules! visit_verify_error{
            ($e : expr, $ty : path, $s : expr) => {
                {
                    self.visit_expr($e);
                    if self.ty != $ty{
                        panic!($s);
                    }
                }
            }
        }
        match *expr{
            //FIXME remove NilExpr; this is only for unit testing
            NilExpr => self.ty = TString,
            NumExpr(_) => self.ty = TInt32,
            StringExpr(_) => self.ty = TString,
            IdExpr(ref mut id) =>{
                //search in the symtab for id's existence and get the type
                let mut found = false;
                for &(ref _id, ref _binding) in &self.sym_tab{ //iterator returns a ref to tuple while iterating; so &(_,_) has to be used
                    if *_id == *id{
                        found = true;
                        self.ty = match **_binding.as_ref().unwrap(){
                            TypeBinding(ref ty) |
                            VarBinding(ref ty) |
                            FuncBinding(ref ty) => ty.clone()
                        };
                        break;
                    }
                }
                if !found{
                    panic!("Invalid reference to variable '{0}'", id);
                }
            },
            LessThanExpr(ref mut e1, ref mut e2) |
            GreaterThanExpr (ref mut e1, ref mut e2) => {
                self.visit_expr(e1) ;
                let lhs_ty = self.ty.clone();
                self.visit_expr(e2);
                if self.ty != lhs_ty && (self.ty != TInt32 || self.ty != TString){
                    panic!("Both types of a relational operator must match and be of type int or string.");
                }
            },
            AddExpr(ref mut left, ref mut right) |
            MulExpr(ref mut left, ref mut  right) => {
                visit_verify_error!(left, TInt32, "Expected left operand of int type");
                visit_verify_error!(right, TInt32, "Expected right operand of int type");
            },
            DivExpr(ref mut left, ref mut  right) => {
                visit_verify_error!(left, TInt32, "Expected left operand of int type");
                visit_verify_error!(right, TInt32, "Expected right operand of int type");
                if let Expr::NumExpr(n) = **right{
                     if n == 0 {panic!("Denominator cannot be 0")}
                }
            },
            SeqExpr(ref mut opt_expr_list) => {
                if opt_expr_list.is_none(){
                    self.ty = TVoid;
                    return
                }
                for b_expr in opt_expr_list.as_mut().unwrap() {
                    self.visit_expr(&mut *b_expr);
                }
            },
            IfThenElseExpr(ref mut conditional_expr, ref mut then_expr, ref mut else_expr) => {
                visit_verify_error!(conditional_expr, TInt32, "Expected conditional expression of int type");

                self.visit_expr(then_expr);
                let then_ty = self.ty.clone();
                self.visit_expr(else_expr);
                if then_ty != self.ty{
                    panic!("Expected then expr and else expr types to be same");
                }
            },
            IfThenExpr(ref mut conditional_expr, ref mut then_expr) => {
                visit_verify_error!(conditional_expr, TInt32, "Expected conditional expression of int type");
                visit_verify_error!(then_expr, TVoid, "Expected if-body of void type");
            },
            WhileExpr(ref mut conditional_expr, ref mut body) => {
                visit_verify_error!(conditional_expr, TInt32, "Expected conditional expression of int type");
                visit_verify_error!(body, TVoid, "Expected while-body of void type");
            },
            ForExpr(_, ref mut from, ref mut to, ref mut body) => {
                visit_verify_error!(from, TInt32, "Initializing expression type should be int in a for loop");
                visit_verify_error!(to, TInt32, "To expression type should be int in a for loop");
                visit_verify_error!(body, TVoid, "A for expression's body must be of type void");
            },
            CallExpr(ref id, ref mut optional_ty_expr_list) => {
                //check if this is a built-in function
                let mut found = false;
                if self.std_functions.contains_key(id){
                    found = true;
                    self.ty = if let Some(&FuncBinding(ref ty)) = self.std_functions.get(id){
                        ty.clone()
                    }
                    else{
                        TNil
                    }
                }
                else{
                    for &(ref _id, ref binding) in self.sym_tab.iter().rev(){
                        if *_id == *id{
                            found = true;
                            if let FuncBinding(ref _ty) = **binding.as_ref().unwrap(){
                                self.ty = _ty.clone();
                                break;
                            }
                        }
                    }
                }
                if !found{
                    panic!("Invalid call to '{0}'. Function not found.", *id);
                }
                //fix call expr return type by doing a sym-tab lookup 
                if optional_ty_expr_list.is_some(){
                    for &mut (ref mut ty, ref mut expr) in optional_ty_expr_list.as_mut().unwrap(){
                        match **expr{
                            CallExpr(ref id, _) => {
                                found = false;
                                for &(ref _id, ref binding) in self.sym_tab.iter().rev(){
                                    if *id == *_id{
                                        if let FuncBinding(ref _ty) = **binding.as_ref().unwrap(){
                                                found = true;
                                                *ty = _ty.clone();
                                                break;
                                        }
                                    }
                                }
                                if !found && self.std_functions.contains_key(id){
                                    *ty = match *self.std_functions.get(id).unwrap(){
                                        FuncBinding(ref ty) => ty.clone(),
                                        _ => {TNil}
                                    }
                                }
                                //FIXME do we panic if function not found?
                            }, 
                            SubscriptExpr(ref id, _) |
                            IdExpr(ref id) => {
                                for &(ref _id, ref binding) in self.sym_tab.iter().rev(){
                                    if *id == *_id{
                                        if let VarBinding(ref _ty) = **binding.as_ref().unwrap(){
                                            *ty = _ty.clone();
                                            break; 
                                        } 
                                    }
                                }
                                
                            },
                            _ => {}
                        }
                    }
                }
            },
            LetExpr(ref mut decls, ref mut opt_expr) => {
                self.sym_tab.push(("<marker>".to_string(), None));

                for dec in decls{ //decls is a &
                    self.visit_decl(dec);
                }

                if let Some(ref mut b_expr) = *opt_expr {
                    self.visit_expr(&mut *b_expr);
                }
                //pop till marker and then pop marker
                while self.sym_tab.last().unwrap().0 != "<marker>"{
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
            VarDec(ref id, ref ty, ref mut expr) => {
                match **expr{
                    IdExpr(ref id) => {
                        for &(ref sym, ref binding) in self.sym_tab.iter().rev(){
                            if *id == *sym{
                                match **binding.as_ref().unwrap(){
                                    VarBinding(_) => {},
                                    _ => panic!(format!("Invalid reference to variable '{0}'. Different binding found.", *id))
                                }
                            }

                        }
                    },
                    ArrayExpr(ref _ty, ref mut _dim_expr, ref mut _init_expr) => {
                        self.visit_expr(&mut *_dim_expr);
                        if *_ty != self.ty{
                            panic!("Array type doesn't match with the type of the dimension expression")
                        }

                        self.visit_expr(&mut *_init_expr);
                        if *_ty != self.ty{
                            panic!("Array type doesn't match with the type of the init expression")
                        }

                        store_into_sym_tab!(self, id, VarBinding);
                        return;
                    },
                    RecordExpr(ref field_decls) => {
                        let list = field_decls.as_ref().unwrap();
                        if !list.is_empty(){
                            let len = list.len();
                            let unique_len = list.into_iter().map(|x| &x.0).unique().count();
                            if len != unique_len{
                                panic!("record '{0}' contains repetitive fields", id);
                            }

                            let rec_contains_cyclic_ref = list.into_iter().find(|x| match x.1{
                                TCustom(ref name) => *id == *name,
                                _ => false
                            });
                            if rec_contains_cyclic_ref.is_some(){
                                panic!("rec '{0}' contains a field of type '{0}'. cyclic references to type are not allowed.", id)
                            } 
                        } 
                    },
                    _ => {}
                }
                self.visit_expr(expr);
                //self.ty can still remain Nil in scenarios
                //where the body contains a call to an  
                //intrinsic function which cannot be verified 
                //by the type-checker
                if self.ty != TNil && *ty != self.ty{
                    panic!(format!("Types mismatch. Variable type is {0} and expression type is '{1}'", *ty, self.ty));
                }
                store_into_sym_tab!(self, id, VarBinding);
            },
            FunDec(ref id, ref params, ref ret_type, ref mut body, ref mut body_type) => {
                self.sym_tab.push((String::from("<marker>"), None));
                if params.is_some(){
                    for p in params.as_ref().unwrap(){
                        self.sym_tab.push((p.0.clone(), Some(B(VarBinding(p.1.clone())))));
                    }
                }
                self.visit_expr(body);
                //self.ty can still remain Nil in scenarios
                //where the body contains a call to an  
                //intrinsic function which cannot be verified 
                //by the type-checker
                if self.ty != TNil && *ret_type != self.ty{
                    panic!(format!("Return type '{0}' doesn't match with the type of the last expression '{1}'.", ret_type, self.ty));
                }
                
                *body_type = self.ty.clone();

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
                //println!("pushing {0}", id);
                self.ty = ret_type.clone();
                store_into_sym_tab!(self, id, FuncBinding);
            },
            TypeDec(ref id, ref ty) => {
                store_into_sym_tab!(self, id, TypeBinding);
            }
        }
    }
}

#[test]
fn test_ty_set_for_num() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut NumExpr(23));
    assert_eq!(TInt32, v.ty);
}

#[test]
fn test_ty_set_for_int_id() {
    let mut v = TypeChecker::new();
    v.sym_tab.push(("a".to_string(), Some(B(VarBinding(TInt32)))));
    v.visit_expr(&mut IdExpr("a".to_string()));
    assert_eq!(TInt32, v.ty);
}

#[test]
fn test_type_match_int_for_var_dec() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TInt32, B(NumExpr(4))));
    assert_eq!(TInt32, v.ty);
    assert_eq!(v.sym_tab.len(), 1);
    assert_eq!(v.sym_tab[0].0, "a".to_string());
    //assert_eq!(v.sym_tab[0].1, TType::TInt32);
}

#[test]
fn test_type_match_string_for_var_dec() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TString, B(NilExpr)));
    assert_eq!(TString, v.ty);
    assert_eq!(v.sym_tab.len(), 1);
    assert_eq!(v.sym_tab[0].0, "a".to_string());
    //assert_eq!(v.sym_tab[0].1, TString);
}

#[test]
fn test_array_type_matches_dim_expr_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TArray(B(TInt32)), B(ArrayExpr(TInt32, B(NumExpr(1)), B(NumExpr(1))))));
}

#[test]
#[should_panic(expected="Array type doesn't match with the type of the dimension expression")]
fn test_array_type_mismatches_dim_expr_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TArray(B(TInt32)), B(ArrayExpr(TString, B(NumExpr(1)), B(NumExpr(1))))));
}

#[test]
#[should_panic(expected="Array type doesn't match with the type of the init expression")]
fn test_array_type_mismatches_init_expr_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TArray(B(TInt32)), B(ArrayExpr(TInt32, B(NumExpr(1)), B(StringExpr(String::from("abhi")))))));
}

#[test]
#[should_panic]
fn test_type_check_for_var_dec_type_mismatch() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TInt32, B(NilExpr)));
    assert_eq!(TInt32, v.ty);
}

#[test]
fn test_correct_types_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut AddExpr(B(NumExpr(4)), B(NumExpr(4))));
    assert_eq!(v.ty, TInt32);
}

#[test]
#[should_panic(expected="Expected left operand of int type")]
fn test_left_type_invalid_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut AddExpr(B(NilExpr), B(NumExpr(4))));
}

#[test]
#[should_panic(expected="Expected right operand of int type")]
fn test_right_type_invalid_for_add_expr() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut AddExpr(B(NumExpr(4)), B(NilExpr)));
}

#[test]
fn test_var_hiding() {
    //let mut v =
}

#[test]
fn test_func_decl_correct_return_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut FunDec(String::from("foo"), None, TInt32, B(NumExpr(4)), TInt32));
}

#[test]
#[should_panic(expected="Return type 'String' doesn't match with the type of the last expression 'Number'.")]
fn test_func_decl_incorrect_return_type() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut FunDec(String::from("foo"), None, TString, B(NumExpr(4)), TInt32));
}

#[test]
#[should_panic(expected="Expected conditional expression of int type")]
fn test_if_expr_with_incorrect_conditional_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut IfThenExpr(B(StringExpr(String::from("a"))), B(StringExpr(String::from("a")))));
}

#[test]
fn test_if_expr_with_int_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut IfThenExpr(B(NumExpr(1)), B(SeqExpr(None))));
    assert_eq!(v.ty, TVoid);
}

#[test]
#[should_panic(expected="Expected if-body of void type")]
fn test_if_expr_with_int_type_conditional_and_int_type_as_body_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut IfThenExpr(B(NumExpr(1)), B(NumExpr(1))));
    assert_eq!(v.ty, TInt32);
}

#[test]
fn test_if_else_expr_with_matching_types() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut IfThenElseExpr(B(NumExpr(1)), B(NumExpr(1)), B(NumExpr(1))));
    assert_eq!(v.ty, TType::TInt32);
}

#[test]
#[should_panic(expected="Expected then expr and else expr types to be same")]
fn test_if_else_expr_with_non_matching_types() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut IfThenElseExpr(B(NumExpr(1)), B(NumExpr(1)), B(StringExpr(String::from("a")))));
}

#[test]
#[should_panic(expected="Expected conditional expression of int type")]
fn test_while_expr_with_incorrect_conditional_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut WhileExpr(B(StringExpr(String::from("a"))), B(StringExpr(String::from("a")))));
}

#[test]
#[should_panic(expected="Expected while-body of void type")]
fn test_while_expr_with_int_type() {
    let mut v = TypeChecker::new();
    v.visit_expr(&mut WhileExpr(B(NumExpr(1)), B(StringExpr(String::from("a")))));
}

#[test]
#[should_panic(expected="Denominator cannot be 0")]
fn test_div_expr_with_0_as_denominator(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut DivExpr(B(NumExpr(1)), B(NumExpr(0))));
}

#[test]
fn test_div_expr_with_1_as_denominator(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut DivExpr(B(NumExpr(1)), B(NumExpr(1))));
    assert_eq!(v.ty, TType::TInt32);
}

#[test]
#[should_panic(expected="Initializing expression type should be int in a for loop")]
fn test_for_loop_expr_init_type(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut ForExpr(String::from("i"),
                                B(StringExpr(String::from("adsd"))),
                                B(NumExpr(1)),
                                B(NumExpr(2))));
}

#[test]
#[should_panic(expected="Duplicate param 'a' found")]
fn test_func_dec_with_duplicate_param_with_same_type(){
    let mut v = TypeChecker::new();
    v.visit_decl(&mut FunDec(String::from("foo"), 
                               Some(vec![(String::from("a"), TInt32),
                                         (String::from("a"), TInt32) ]),
                               TInt32,
                               B(NumExpr(4)),
                               TInt32));
}

#[test]
#[should_panic(expected="Duplicate param 'a' found")]
fn test_func_dec_with_duplicate_param_with_different_types(){
    let mut v = TypeChecker::new();
    v.visit_decl(&mut FunDec(String::from("foo"), 
                               Some(vec![(String::from("a"), TInt32),
                                         (String::from("a"), TString) ]),
                               TInt32,
                               B(NumExpr(4)),
                               TInt32));
}

#[test]
fn test_call_expr_call_print_with_params(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut CallExpr(String::from("print"),
                                     Some(vec![(TString, B(StringExpr(String::from("abhi"))))])));
    assert_eq!(v.ty, TVoid);
}

#[test]
fn test_call_expr_call_not_with_params(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut CallExpr(String::from("not"),
                                     Some(vec![(TInt32, B(NumExpr(0)))])));
    assert_eq!(v.ty, TInt32);
}

#[test]
#[should_panic(expected="Invalid call to 'foo'. Function not found.")]
fn test_call_expr_call_undefined_function(){
    let mut v = TypeChecker::new();
    v.visit_expr(&mut CallExpr(String::from("foo"), None));
}

#[test]
fn test_type_fix_func_return_type(){
    let mut v = TypeChecker::new();
    let dec = &mut FunDec(String::from("foo"), 
                               Some(vec![(String::from("a"), TInt32)
                                          ]),
                               TInt32,
                               B(NumExpr(4)),
                               TVoid);
    v.visit_decl(dec);
    match *dec{
        FunDec(_, _, _, _, ref ty) => assert_eq!(TInt32, *ty),
        _ => panic!("Expected FunDec")
    }
}
#[test]
fn test_call_expr_ret_type_fix(){
    let mut v = TypeChecker::new();
    let e = &mut LetExpr(vec![FunDec(String::from("foo"), 
                               Some(vec![(String::from("a"), TInt32)]),
                               TInt32,
                               B(NumExpr(4)),
                               TInt32)],
                               Some(B(CallExpr(String::from("foo"),
                                                     Some(vec![(TVoid, B(CallExpr(String::from("foo"),
                                                                                               Some(vec![(TInt32, B(NumExpr(2)))])
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
        LetExpr(_, ref e) => {
            match **e.as_ref().unwrap(){
                CallExpr(_, ref l) => {
                    let ul = l.as_ref().unwrap();
                    match ul[0]{
                        (TInt32, _) => {},
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

#[test]
#[should_panic(expected="record 'a' contains repetitive fields")]
fn test_record_dup_fields_1() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TRecord, B(RecordExpr(Some(vec![(String::from("f"), TInt32), (String::from("f"), TInt32)])))));
}

#[test]
#[should_panic(expected="record 'a' contains repetitive fields")]
fn test_record_dup_fields_2() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TRecord, B(RecordExpr(Some(vec![(String::from("f"), TInt32), (String::from("g"), TInt32), (String::from("f"), TInt32)])))));
}

#[test]
#[should_panic(expected="record 'a' contains repetitive fields")]
fn test_record_dup_fields_3() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TRecord, B(RecordExpr(Some(vec![(String::from("f"), TInt32), (String::from("g"), TInt32), (String::from("f"), TString)])))));
}

#[test]
fn test_record_unique_fields() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TRecord, B(RecordExpr(Some(vec![(String::from("f"), TInt32), (String::from("g"), TInt32), (String::from("h"), TString)])))));
}

#[test]
#[should_panic(expected="rec 'a' contains a field of type 'a'. cyclic references to type are not allowed.")]
fn test_record_contains_cyclic_ref() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TRecord, B(RecordExpr(Some(vec![(String::from("f"), TCustom(String::from("a")))])))));
}

#[test]
#[should_panic(expected="rec 'a' contains a field of type 'a'. cyclic references to type are not allowed.")]
fn test_record_contains_cyclic_ref_2() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TRecord, B(RecordExpr(Some(vec![(String::from("a"), TCustom(String::from("int"))),
                                                                              (String::from("f"), TCustom(String::from("a")))])))));
}

#[test]
fn test_record_contains_cyclic_ref_3() {
    let mut v = TypeChecker::new();
    v.visit_decl(&mut VarDec("a".to_string(), TRecord, B(RecordExpr(Some(vec![(String::from("a"), TCustom(String::from("int"))),
                                                                              (String::from("f"), TCustom(String::from("b")))])))));
}

#[test]
fn test_record_field_access_type_fix(){
    let mut v = TypeChecker::new();
    //let var a = rec{b:int} in print(a.b) end
    let e = &mut LetExpr(vec![VarDec("a".to_string(), TRecord, B(RecordExpr(Some(vec![(String::from("f"), TInt32), (String::from("g"), TInt32), (String::from("h"), TString)]))))],
                               Some(B(CallExpr(String::from("foo"),
                                                 Some(vec![(TNil, B(FieldExpr(String::from("a"),
                                                                              String::from("f")
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
        LetExpr(_, ref e) => {
            match **e.as_ref().unwrap(){
                CallExpr(_, ref l) => {
                }
                _ => {}
            }
        }
        _ => {}
    }
}
