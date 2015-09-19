#![feature(rustc_private)]
extern crate llvm_sys as llvm;
extern crate libc;
use std::ptr;
use std::ffi;

use self::llvm::prelude::{LLVMContextRef, LLVMModuleRef, LLVMBuilderRef, LLVMValueRef};
use self::llvm::core::*;

use std::collections::{HashMap};
use std::mem;

use syntax::ast::{Block, Expr, TType, OptionalTypeExprTupleList};

macro_rules! c_str_ptr {
    ($s:expr) => {
        ffi::CString::new($s).unwrap().as_ptr()
    };
}

pub struct Context{
    context : LLVMContextRef,
    module : LLVMModuleRef,
    builder : LLVMBuilderRef,
    named_values : HashMap<String, LLVMValueRef>
}

impl Context{
    fn new(module_name : &str) -> Self{
        unsafe{
            let llvm_context =  LLVMContextCreate();
            let llvm_module = LLVMModuleCreateWithNameInContext(c_str_ptr!(module_name),
                                                                llvm_context);
            let builder = LLVMCreateBuilderInContext(llvm_context);
            let named_values = HashMap::new();

            Context {
                context : llvm_context,
                module : llvm_module,
                builder : builder,
                named_values : named_values
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
                    &Expr::StringExpr(ref value) => c_str_ptr!(&**value),
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
                &Expr::CallExpr(ref fn_name, ref optional_args) => {
                    match std_functions_call_factory(&*fn_name, optional_args, ctxt) {
                        Some(call) => Ok(call),
                        _ => {Err("Should handle non std functions".to_string())}
                    }
                },
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
        LLVMBuildRet(ctxt.builder,
                     LLVMConstInt(LLVMIntTypeInContext(ctxt.context, 32), 0 as u64, 0));

        //add translated code as part of the block
        trans_expr(expr, &mut ctxt);

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
