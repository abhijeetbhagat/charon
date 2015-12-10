extern crate llvm_sys as llvm;
extern crate libc;
use std::ptr;
use std::ffi;

use self::llvm::prelude::{LLVMContextRef, LLVMModuleRef, LLVMBuilderRef, LLVMValueRef, LLVMTypeRef};
use self::llvm::core::*;
use self::llvm::target::*;
use self::llvm::target_machine::*;

use std::collections::{HashMap};
use std::mem;
use std::any::{Any};
use syntax::ast::{Block, Expr, Decl, TType, OptionalTypeExprTupleList};
use syntax::ptr::{B};
//FIXME this import is for integration testing purposes
use syntax::parse::*;//{Parser};
use syntax::parse::parser::{Parser};
use link::link;
use helpers::*;
use symbol::*;

//FIXME this is only of unit testing
pub type OptionalSymbolInfo = Option<Box<Any>>;

pub struct Context<'a>{
    context : LLVMContextRef,
    pub module : LLVMModuleRef,
    builder : LLVMBuilderRef,
    //FIXME this is only for unit testing
    pub sym_tab : Vec<(String, OptionalSymbolInfo)>,
    bb_stack : Vec<*mut llvm::LLVMBasicBlock>,
    proto_map : HashMap<&'a str, bool>
}

impl<'a> Context<'a>{
    fn new(module_name : &str) -> Self{
        unsafe{
            let llvm_context =  LLVMContextCreate();
            let llvm_module = LLVMModuleCreateWithNameInContext(c_str_ptr!(module_name),
                                                                llvm_context);
            let builder = LLVMCreateBuilderInContext(llvm_context);
            let sym_tab = Vec::new();
            let bb_stack = Vec::new();
            let proto_map = HashMap::new();

            Context {
                context : llvm_context,
                module : llvm_module,
                builder : builder,
                sym_tab : sym_tab,
                bb_stack : bb_stack,
                proto_map : proto_map
            }
        }
    }

    pub fn dump(&self){
        unsafe{
            LLVMDumpModule(self.module);
        }
    }
}

impl<'a> Drop for Context<'a>{
    fn drop(&mut self){
        unsafe{
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}

//TODO move these in a seperate file
type IRBuildingResult = Result<LLVMValueRef, String>;

trait IRBuilder{
    fn codegen(&self, ctxt : &mut Context) -> IRBuildingResult;
}

fn std_functions_call_factory(fn_name : &str,
                              args : &OptionalTypeExprTupleList,
                              ctxt : &mut Context) -> Option<LLVMValueRef>{
    unsafe{
        match fn_name {
            "print" =>{
                debug_assert!(args.is_some(), "No args passed to print()");
                let lst = args.as_ref().unwrap();
                debug_assert!(lst.len() == 1, "One arg should be passed to print()");
                debug_assert!(lst[0].0 == TType::TString || lst[0].0 == TType::TInt32);
                let str_arg = match &*lst[0].1 {
                    &Expr::StringExpr(ref value) => c_str_ptr!(&*(value.clone())),
                    _ => panic!("Expected a string expr")
                };

                let print_function : LLVMValueRef;
                //check if we already have a prototype defined
                if !ctxt.proto_map.contains_key("printf"){
                    let print_ty = LLVMIntTypeInContext(ctxt.context, 32);
                    let mut pf_type_args_vec = Vec::new();
                    pf_type_args_vec.push(LLVMPointerType(LLVMIntTypeInContext(ctxt.context, 8),
                    0));
                    let proto = LLVMFunctionType(print_ty, pf_type_args_vec.as_mut_ptr(), 1, 1);
                    print_function = LLVMAddFunction(ctxt.module,
                                                         c_str_ptr!("printf"),
                                                         proto);
                    ctxt.proto_map.insert("printf", true);
                }
                else{
                    print_function = LLVMGetNamedFunction(ctxt.module, c_str_ptr!("printf"));
                }

                let gstr = LLVMBuildGlobalStringPtr(ctxt.builder,
                                                    str_arg,
                                                    c_str_ptr!(".str"));
                let mut pf_args = Vec::new();
                pf_args.push(gstr);

                Some(LLVMBuildCall(ctxt.builder,
                                   print_function,
                                   pf_args.as_mut_ptr(),
                                   1,
                                   c_str_ptr!("call")))
            },
            _ => {None}
        }
    }
}

fn get_llvm_type_for_ttype(ty : &TType, ctxt : &mut Context) -> LLVMTypeRef{
    unsafe{
        match ty {
            &TType::TVoid => LLVMVoidTypeInContext(ctxt.context),
            &TType::TInt32 => LLVMIntTypeInContext(ctxt.context, 32),
            _ => panic!("Other TTypes not mapped yet to the corresponding LLVM types")
        }
    }
}

impl IRBuilder for Expr{
    fn codegen(&self, ctxt : &mut Context) -> IRBuildingResult{
        macro_rules! build_binary_instrs{
            ($fun : ident, $e1:ident, $e2:ident, $s : expr) => {{
                let ev1 = try!($e1.codegen(ctxt));
                let ev2 = try!($e2.codegen(ctxt));
                Ok($fun(ctxt.builder, ev1, ev2, $s.as_ptr() as *const i8))
            }}
        }
        unsafe{
            match self{
                &Expr::NumExpr(ref i) => {
                    let ty = LLVMIntTypeInContext(ctxt.context, 32);
                    Ok(LLVMConstInt(ty, *i as u64, 0))
                },
                &Expr::AddExpr(ref e1, ref e2) => {
                    build_binary_instrs!(LLVMBuildAdd, e1, e2, "add_tmp")
                },
                &Expr::SubExpr(ref e1, ref e2) => {
                    build_binary_instrs!(LLVMBuildSub, e1, e2, "sub_tmp")
                },
                &Expr::MulExpr(ref e1, ref e2) => {
                    build_binary_instrs!(LLVMBuildMul, e1, e2, "mul_tmp")
                },
                &Expr::DivExpr(ref e1, ref e2) => {
                    build_binary_instrs!(LLVMBuildSDiv, e1, e2, "div_tmp")
                },
                &Expr::LessThanExpr(ref e1, ref e2) => {
                    let ev1 = try!(e1.codegen(ctxt));
                    let ev2 = try!(e2.codegen(ctxt));
                    Ok(LLVMBuildICmp(ctxt.builder, llvm::LLVMIntPredicate::LLVMIntSLT, ev1, ev2, c_str_ptr!("lecmp_tmp")))
                },
                &Expr::GreaterThanExpr(ref e1, ref e2) => {
                    let ev1 = try!(e1.codegen(ctxt));
                    let ev2 = try!(e2.codegen(ctxt));
                    Ok(LLVMBuildICmp(ctxt.builder, llvm::LLVMIntPredicate::LLVMIntSGT, ev1, ev2, c_str_ptr!("gtcmp_tmp")))
                },
                &Expr::NotEqualsExpr(ref e1, ref e2) => {
                    let ev1 = try!(e1.codegen(ctxt));
                    let ev2 = try!(e2.codegen(ctxt));
                    Ok(LLVMBuildICmp(ctxt.builder, llvm::LLVMIntPredicate::LLVMIntNE, ev1, ev2, c_str_ptr!("necmp_tmp")))
                },
                &Expr::IdExpr(ref id) => {
                    let mut sym = &None;
                    let mut found = false;
                    for &(ref _id, ref info) in ctxt.sym_tab.iter().rev(){
                        if *_id == *id  {
                            sym = info;
                            found = true;
                            break;
                        }
                    }

                    if !found{
                        panic!(format!("Invalid reference to variable '{0}'", *id));
                    }

                    let _optional = sym.as_ref().unwrap().downcast_ref::<Var>();
                    if _optional.is_some(){
                        Ok(_optional.unwrap().alloca_ref())
                    }
                    else{
                        panic!(format!("Invalid reference to variable '{0}'. Different binding found.", *id));
                    }
                },
                &Expr::IfThenElseExpr(ref conditional_expr, ref then_expr, ref else_expr) => {
                    let cond_code = try!(conditional_expr.codegen(ctxt));
                    let zero = LLVMConstInt(LLVMIntTypeInContext(ctxt.context, 32), 0u64, 0);
                    let if_cond = LLVMBuildICmp(ctxt.builder, llvm::LLVMIntPredicate::LLVMIntNE, cond_code, zero, c_str_ptr!("ifcond"));
                    let bb = LLVMGetInsertBlock(ctxt.builder);
                    let function = LLVMGetBasicBlockParent(bb);
                    let then_block = LLVMAppendBasicBlockInContext(ctxt.context, function, c_str_ptr!("thencond"));
                    let else_block = LLVMAppendBasicBlockInContext(ctxt.context, function, c_str_ptr!("elsecond"));
                    let ifcont_block = LLVMAppendBasicBlockInContext(ctxt.context, function, c_str_ptr!("ifcont"));
                    LLVMBuildCondBr(ctxt.builder, if_cond, then_block, else_block); 

                    LLVMPositionBuilderAtEnd(ctxt.builder, then_block);
                    let then_code = try!(then_expr.codegen(ctxt));
                    LLVMBuildBr(ctxt.builder, ifcont_block);
                    let then_end = LLVMGetInsertBlock(ctxt.builder);

                    LLVMPositionBuilderAtEnd(ctxt.builder, else_block);
                    let else_code = try!(else_expr.codegen(ctxt));
                    LLVMBuildBr(ctxt.builder, ifcont_block);
                    let else_end = LLVMGetInsertBlock(ctxt.builder);

                    LLVMPositionBuilderAtEnd(ctxt.builder, ifcont_block);

                    let phi_node = LLVMBuildPhi(ctxt.builder, LLVMIntTypeInContext(ctxt.context, 32), c_str_ptr!("ifphi"));
                    LLVMAddIncoming(phi_node, vec![then_code].as_mut_ptr(), vec![then_end].as_mut_ptr(), 1);
                    LLVMAddIncoming(phi_node, vec![else_code].as_mut_ptr(), vec![else_end].as_mut_ptr(), 1);
                    Ok(phi_node)

                },
                &Expr::ForExpr(ref id, ref from, ref to, ref do_expr) => {
                    assert!(!id.is_empty(), "id cannot be empty");
                    let from_code = try!(from.codegen(ctxt));
                    let bb = LLVMGetInsertBlock(ctxt.builder);
                    let function = LLVMGetBasicBlockParent(bb);

                    //i := ...
                    let from_var = LLVMBuildAlloca(ctxt.builder, LLVMIntTypeInContext(ctxt.context, 32), c_str_ptr!(&*id.clone()));
                    let store = LLVMBuildStore(ctxt.builder, from_code, from_var);

                    let preloop_block = LLVMAppendBasicBlockInContext(ctxt.context, function, c_str_ptr!("preloop"));
                    LLVMBuildBr(ctxt.builder, preloop_block);
                    LLVMPositionBuilderAtEnd(ctxt.builder, preloop_block);

                    let to_code = try!(to.codegen(ctxt));
                    let zero = LLVMConstInt(LLVMIntTypeInContext(ctxt.context, 32), 0 as u64, 0);
                    let end_cond = LLVMBuildICmp(ctxt.builder,
                                                 llvm::LLVMIntPredicate::LLVMIntNE,
                                                 LLVMBuildLoad(ctxt.builder, from_var, c_str_ptr!(&*id.clone())),
                                                 to_code,
                                                 c_str_ptr!("loopcond"));

                    let afterloop_block = LLVMAppendBasicBlockInContext(ctxt.context, function, c_str_ptr!("afterloop"));
                    let loop_block = LLVMAppendBasicBlockInContext(ctxt.context, function, c_str_ptr!("loop"));
                    LLVMBuildCondBr(ctxt.builder, end_cond, loop_block, afterloop_block);
                    
                    LLVMPositionBuilderAtEnd(ctxt.builder, loop_block);
                    let do_expr_code = try!(do_expr.codegen(ctxt));

                    //stepping
                    let cur_value = LLVMBuildLoad(ctxt.builder, from_var, c_str_ptr!(&*id.clone()));
                    let next_value = LLVMBuildAdd(ctxt.builder, cur_value, 
                                                  LLVMConstInt(LLVMIntTypeInContext(ctxt.context, 32), 1 as u64, 0), 
                                                  c_str_ptr!("nextvar"));
                    LLVMBuildStore(ctxt.builder, next_value, from_var);

                    LLVMBuildBr(ctxt.builder, preloop_block);
                    LLVMPositionBuilderAtEnd(ctxt.builder, afterloop_block);

                    //FIXME remove this 
                    Ok(zero)
                },
                //&Expr::WhileExpr(ref conditional_expr, ref body) => {
                    //let cond_code = try!(conditional_expr.codegen(ctxt));

                ////},
                &Expr::CallExpr(ref fn_name, ref optional_args) => {
                    //FIXME instead of directly passing to the factory
                    //fn_name can be checked in a map that records names of std functions
                    match std_functions_call_factory(&*fn_name, optional_args, ctxt) {
                        Some(call) => Ok(call), //intrinsic function
                        _ => { //user-defined function
                            //user defined function call
                            let mut pf_args = Vec::new();
                            //FIXME pass args if present in the call
                            if optional_args.is_some() {

                            }
                            
                            let mut sym = &None;
                            let mut found = false;
                            for &(ref id, ref info) in ctxt.sym_tab.iter().rev(){
                                if *id == *fn_name{
                                    sym = info;
                                    found = true;
                                    break;
                                }
                            }

                            if !found{
                                panic!(format!("Call to '{0}' not found", fn_name));
                            }

                            let _optional = sym.as_ref().unwrap().downcast_ref::<Function>();
                            if _optional.is_some(){
                                Ok(LLVMBuildCall(ctxt.builder,
                                            _optional.as_ref().unwrap().value_ref(),
                                            pf_args.as_mut_ptr(),
                                            0,
                                            c_str_ptr!("")))
                            }
                            else{
                                panic!(format!("Invalid reference to function '{0}'. Different binding found.", *fn_name));
                            }
                        }
                    }
                },
                &Expr::LetExpr(ref decls, ref expr) => {
                    debug_assert!(!decls.is_empty(), "Declarations in a let block can't be empty");
                    debug_assert!(expr.is_some(), "Expr in a let block can't be empty");
                    for decl in &*decls {
                        match decl {
                            &Decl::FunDec(ref name, ref params, ref ty, ref body, ref body_ty) => {
                                let llvm_ty = get_llvm_type_for_ttype(ty, ctxt);
                                let proto = LLVMFunctionType(llvm_ty, ptr::null_mut(), 0, 0);
                                let cloned_name = name.clone();
                                let function = LLVMAddFunction(ctxt.module,
                                                               c_str_ptr!(&(*cloned_name)),
                                                               proto);
                                let bb = LLVMAppendBasicBlockInContext(ctxt.context,
                                                                       function,
                                                                       c_str_ptr!("entry"));

                                let func = Function::new(cloned_name.clone(), function);
                                ctxt.sym_tab.push((cloned_name.clone(), Some(Box::new(func))));
                                LLVMPositionBuilderAtEnd(ctxt.builder, bb);
                                //trans_expr(body, &mut ctxt);
                                let value_ref = try!(body.codegen(ctxt));
                                if *ty == TType::TVoid{
                                    LLVMBuildRetVoid(ctxt.builder);
                                }
                                else{
                                    LLVMBuildRet(ctxt.builder, value_ref);
                                }
                            },

                            &Decl::VarDec(ref name, ref ty, ref rhs) => {
                                let llvm_ty = get_llvm_type_for_ttype(ty, ctxt);
                                let alloca = LLVMBuildAlloca(ctxt.builder, llvm_ty, c_str_ptr!(&(*name.clone())));
                                let rhs_value_ref = try!(rhs.codegen(ctxt));
                                let store = LLVMBuildStore(ctxt.builder,
                                                           rhs_value_ref,
                                                           alloca);
                                ctxt.sym_tab.push((name.clone(), Some(Box::new(Var::new(name.clone(), ty.clone(), alloca)))));
                            },
                            _ => panic!("More decl types should be covered")
                        }

                    }
                    
                    //translation of the 'in' expr
                    
                    //trans_expr(&*expr.unwrap(), &mut ctxt);
                    //FIXME should the previous bb be popped here?
                    let bb = ctxt.bb_stack.pop().unwrap();
                    LLVMPositionBuilderAtEnd(ctxt.builder, bb);
                    let e = &expr.as_ref().unwrap();
                    let v = try!(e.codegen(ctxt));
                    //pop all the symbols declared in the current let block
                    Ok(v)
                }
                t => Err(format!("error: {:?}", t))
            }
        }
    }
}

pub fn translate(expr : &Expr) -> Option<Context>{
    let mut ctxt = Context::new("main_mod");
    unsafe{
        let r = LLVM_InitializeNativeTarget();
        assert_eq!(r, 0);
        LLVM_InitializeNativeAsmPrinter();
        //build outer embedding main() fn
        let ty = LLVMIntTypeInContext(ctxt.context, 32);
        let proto = LLVMFunctionType(ty, ptr::null_mut(), 0, 0);
        let function = LLVMAddFunction(ctxt.module,
                                       c_str_ptr!("main"),
                                       proto);
        let bb = LLVMAppendBasicBlockInContext(ctxt.context,
                                               function,
                                               c_str_ptr!("entry"));
        LLVMPositionBuilderAtEnd(ctxt.builder, bb);
        ctxt.bb_stack.push(bb);
        trans_expr(expr, &mut ctxt);
        
        LLVMBuildRet(ctxt.builder,
                     LLVMConstInt(LLVMIntTypeInContext(ctxt.context, 32), 0 as u64, 0));

    }
    Some(ctxt)
}

fn link_object_code(ctxt : &Context){
    link(ctxt);
}

fn trans_expr(expr: &Expr, ctxt : &mut Context){
    let result = expr.codegen(ctxt); //ctxt is already &mut
    match result {
        Ok(_) => {},
        Err(msg) => panic!(msg)
    }
}

#[test]
fn test_translate_std_print_call() {
    let expr = &Expr::CallExpr("print".to_string(),
                                  Some(vec![(TType::TString,
                                             B(Expr::StringExpr("abhi".to_string())))]));
    let ctxt = translate(expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_prnt_call() {
    let mut p = Parser::new("print(\"Grrrr!\n\")".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    // let ctxt = translate();&Expr::CallExpr("print".to_string(),
    //                               Some(vec![(TType::TString,
    //                                          B(Expr::StringExpr("abhi".to_string())))])));
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_translate_add_expr(){
    let mut p = Parser::new(String::from("let function foo() : int = 1+3 in foo() end"));
    p.start_lexer();
    let tup = p.expr();
    let (_ , b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
}
#[test]
fn test_prsr_bcknd_intgrtion_let_blk() {
    let mut p = Parser::new("let function foo() = print(\"Grrrr!\n\") in foo() end".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_if_then_expr() {
    let mut p = Parser::new("let function foo()  = if 0 then print(\"rust\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_if_then_expr_with_div_expr() {
    let mut p = Parser::new("let function foo()  = if 1/1 then print(\"rust\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_if_then_expr_with_mul_expr() {
    let mut p = Parser::new("let function foo()  = if 1*1 then print(\"ruby\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_if_then_expr_with_less_than_expr() {
    let mut p = Parser::new("let function foo() = if 1<1 then print(\"ruby\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}
#[test]
fn test_prsr_bcknd_intgrtion_var_decl() {
    let mut p = Parser::new("let var a : int :=1\n function foo()  = print(\"ruby\n\") in foo() end".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_for_loop() {
    let mut p = Parser::new("let function foo() = for i:=1 to 5 do print(\"ruby\n\") in foo() end".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
#[should_panic(expected="Call to 'foo' not found")]
fn test_prsr_bcknd_intgrtion_invalid_call() {
    let mut p = Parser::new("foo()".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
}


#[test]
#[should_panic(expected="Invalid reference to variable 'i'")]
fn test_prsr_bcknd_intgrtion_invalid_reference_to_var() {
    let mut p = Parser::new("let var a : int :=i in foo()".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
}

#[test]
fn test_prsr_bcknd_intgrtion_var_assignment_to_var() {
    let mut p = Parser::new("let var i : int := 1\nvar a : int :=i in print(\"\")".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.unwrap().sym_tab.len(), 2);
}

#[test]
#[should_panic(expected="Invalid reference to variable 'foo'. Different binding found.")]
fn test_prsr_bcknd_intgrtion_invalid_reference_to_var_defined_as_function() {
    let mut p = Parser::new("let function foo() = print(\"b\")\nvar i : int := foo\n in print(\"\")".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
}

#[test]
#[should_panic(expected="Invalid reference to function 'foo'. Different binding found.")]
fn test_prsr_bcknd_intgrtion_invalid_reference_to_func_defined_as_var() {
    let mut p = Parser::new("let var foo : int := 1\n in foo()".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
}
//#[test]
//fn test_prsr_bcknd_intgrtion_for_expr() {
//    let mut p = Parser::new("let function foo() = for i:= 1 to 1 do 1 in foo() end".to_string());
//    p.start_lexer();
//    let tup = p.expr();
//    let (ty, b_expr) = tup.unwrap();
//    let ctxt = translate(&*b_expr);
//    assert_eq!(ctxt.is_some(), true);
//    ctxt.unwrap().dump();
//}
