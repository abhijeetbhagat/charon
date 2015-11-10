#![feature(rustc_private)]
extern crate llvm_sys as llvm;
extern crate libc;
use std::ptr;
use std::ffi;

use self::llvm::prelude::{LLVMContextRef, LLVMModuleRef, LLVMBuilderRef, LLVMValueRef, LLVMTypeRef};
use self::llvm::core::*;

use std::collections::{HashMap};
use std::mem;

use syntax::ast::{Block, Expr, Decl, TType, OptionalTypeExprTupleList};
use syntax::ptr::{B};
//FIXME this import is for integration testing purposes
use syntax::parse::*;//{Parser};
use syntax::parse::parser::{Parser};

macro_rules! c_str_ptr {
    ($s:expr) => {
        ffi::CString::new($s).unwrap().as_ptr()
    };
}

pub struct Context{
    context : LLVMContextRef,
    module : LLVMModuleRef,
    builder : LLVMBuilderRef,
    sym_tab : HashMap<String, LLVMValueRef>,
    bb_stack : Vec<*mut llvm::LLVMBasicBlock>
}

impl Context{
    fn new(module_name : &str) -> Self{
        unsafe{
            let llvm_context =  LLVMContextCreate();
            let llvm_module = LLVMModuleCreateWithNameInContext(c_str_ptr!(module_name),
                                                                llvm_context);
            let builder = LLVMCreateBuilderInContext(llvm_context);
            let sym_tab = HashMap::new();
            let bb_stack = Vec::new();

            Context {
                context : llvm_context,
                module : llvm_module,
                builder : builder,
                sym_tab : sym_tab,
                bb_stack : bb_stack
            }
        }
    }

    pub fn dump(&self){
        unsafe{
            LLVMDumpModule(self.module);
        }
    }
}

impl Drop for Context{
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
                debug_assert!(lst[0].0 == TType::TString);
                let str_arg = match &*lst[0].1 {
                    &Expr::StringExpr(ref value) => c_str_ptr!(&*(value.clone())),
                    _ => panic!("Expected a string expr")
                };

                let print_ty = LLVMIntTypeInContext(ctxt.context, 32);
                let mut pf_type_args_vec = Vec::new();
                pf_type_args_vec.push(LLVMPointerType(LLVMIntTypeInContext(ctxt.context, 8),
                                                      0));
                let proto = LLVMFunctionType(print_ty, pf_type_args_vec.as_mut_ptr(), 1, 1);
                let print_function = LLVMAddFunction(ctxt.module,
                                                     c_str_ptr!("printf"),
                                                     proto);
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
        unsafe{
            match self{
                &Expr::NumExpr(ref i) => {
                    let ty = LLVMIntTypeInContext(ctxt.context, 32);
                    Ok(LLVMConstInt(ty, *i as u64, 0))
                },
                &Expr::AddExpr(ref e1, ref e2) => {
                    let ev1 = try!(e1.codegen(ctxt));
                    let ev2 = try!(e2.codegen(ctxt));
                    Ok(LLVMBuildFAdd(ctxt.builder, ev1, ev2, "add_tmp".as_ptr() as *const i8))
                },
                &Expr::SubExpr(ref e1, ref e2) => {
                    let ev1 = try!(e1.codegen(ctxt));
                    let ev2 = try!(e2.codegen(ctxt));
                    Ok(LLVMBuildFSub(ctxt.builder, ev1, ev2, "sub_tmp".as_ptr() as *const i8))
                },
                &Expr::CallExpr(ref fn_name, ref optional_args) => {
                    //FIXME instead of directly passing to the factory
                    //fn_name can be checked in a map that records names of std functions
                    match std_functions_call_factory(&*fn_name, optional_args, ctxt) {
                        Some(call) => Ok(call),
                        _ => {
                            //user defined function call
                            let mut pf_args = Vec::new();
                            //FIXME pass args if present in the call
                            if optional_args.is_some() {

                            }
                            //pf_args.push(gstr);
                            let _fn = ctxt.sym_tab[&*fn_name];
                            Ok(LLVMBuildCall(ctxt.builder,
                                            _fn,
                                            pf_args.as_mut_ptr(),
                                            0,
                                            c_str_ptr!("")))
                        }
                    }
                },
                &Expr::LetExpr(ref decls, ref expr) => {
                    debug_assert!(!decls.is_empty(), "Declarations in a let block can't be empty");
                    debug_assert!(expr.is_some(), "Expr in a let block can't be empty");
                    for decl in &*decls {
                        match decl {
                            &Decl::FunDec(ref name, ref params, ref ty, ref body) => {
                                let llvm_ty = get_llvm_type_for_ttype(ty, ctxt);
                                let proto = LLVMFunctionType(llvm_ty, ptr::null_mut(), 0, 0);
                                let function = LLVMAddFunction(ctxt.module,
                                                               c_str_ptr!(&(*name.clone())),
                                                               proto);
                                let bb = LLVMAppendBasicBlockInContext(ctxt.context,
                                                                       function,
                                                                       c_str_ptr!("entry"));
                                ctxt.sym_tab.insert(name.clone(), function);
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
                            _ => panic!("More decl types should be covered")
                        }

                    }
                    //trans_expr(&*expr.unwrap(), &mut ctxt);
                    //FIXME should the previous bb be popped here?
                    let bb = ctxt.bb_stack.pop().unwrap();
                    LLVMPositionBuilderAtEnd(ctxt.builder, bb);
                    let e = &expr.as_ref().unwrap();
                    let v = try!(e.codegen(ctxt));
                    Ok(v)
                }
                _ => Err("error".to_string())
            }
        }
    }
}

pub fn translate(expr : &Expr) -> Option<Context>{
    let mut ctxt = Context::new("main_mod");
    unsafe{
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

        //add translated code as part of the block
    }
    Some(ctxt)
}

fn trans_expr(expr: &Expr, ctxt : &mut Context){
    let result = expr.codegen(ctxt); //ctxt is already &mut
    match result {
        Ok(_) => {},
        Err(msg) => panic!(msg)
    }
}

//#[test]
fn test_translate_std_print_call() {
    let ctxt = translate(&Expr::CallExpr("print".to_string(),
                                  Some(vec![(TType::TString,
                                             B(Expr::StringExpr("abhi".to_string())))])));
    assert_eq!(ctxt.is_some(), true);
    ctxt.unwrap().dump();
}

//#[test]
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
    ctxt.unwrap().dump();
}

#[test]
fn test_translate_add_expr(){
    let mut p = Parser::new(String::from("let function foo() : int = 1+3 in foo() end"));
    p.start_lexer();
    let tup = p.expr();
    let (_ , b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    ctxt.unwrap().dump();
}
#[test]
fn test_prsr_bcknd_intgrtion_let_blk() {
    let mut p = Parser::new("let function foo() = print(\"Grrrr!\n\") in foo() end".to_string());
    p.start_lexer();
    let tup = p.expr();
    let (ty, b_expr) = tup.unwrap();
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
    ctxt.unwrap().dump();
}
