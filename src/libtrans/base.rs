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
use syntax::ast::Expr::*;
use syntax::ptr::{B};
//FIXME is this the appropriate place to call the TC?
use syntax::visit::{Visitor};
use syntax::visitor_impl::{TypeChecker};
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

//fn get_result(e : &Expr, ctxt : &mut Context) -> Result<LLVMValueRef, String>{
//    let r = try!(e.codegen(ctxt));
//    r
//}
fn not_builder(ctxt : &mut Context) {
    unsafe{ 
        let not_function : LLVMValueRef;
        //check if we already have a prototype defined
        let not_ty = LLVMIntTypeInContext(ctxt.context, 32);
        let mut not_type_args_vec = vec![LLVMIntTypeInContext(ctxt.context, 32)];
        let proto = LLVMFunctionType(not_ty, not_type_args_vec.as_mut_ptr(), 1, 0);
        not_function = LLVMAddFunction(ctxt.module,
                                       c_str_ptr!("not"),
                                       proto);
        let bb = LLVMAppendBasicBlockInContext(ctxt.context,
                                               not_function,
                                               c_str_ptr!("entry"));

        let func = Function::new(String::from("not"), not_function);
        //FIXME this should be inserted at the beginning to indicate the fact that
        //it belongs to the global scope
        ctxt.sym_tab.push((String::from("not"), Some(Box::new(func))));
        LLVMPositionBuilderAtEnd(ctxt.builder, bb);

        //build allocas for params
        let c = LLVMCountParams(not_function) as usize;
        assert_eq!(c, 1);
        let mut params_vec = Vec::with_capacity(c);
        let p = params_vec.as_mut_ptr();
        mem::forget(params_vec);
        LLVMGetParams(not_function, p);
        let mut v = Vec::from_raw_parts(p, c, c);
        assert_eq!(v.len(), 1);
        //assert_eq!(params_vec.len(), 1);
        let alloca = LLVMBuildAlloca(ctxt.builder,
                                     LLVMIntTypeInContext(ctxt.context, 32),
                                     c_str_ptr!("a"));
        LLVMBuildStore(ctxt.builder,
                       v[0],
                       alloca);
        ctxt.sym_tab.push((String::from("a"), Some(Box::new(Var::new(String::from("a"), TType::TInt32, alloca)))));
        let body = IfThenElseExpr(B(EqualsExpr(B(IdExpr(String::from("a"))), B(NumExpr(0)))),
        B(NumExpr(1)),
        B(NumExpr(0)));
        let value_ref = match body.codegen(ctxt){
            Ok(v_ref) => v_ref,
            Err(e) => panic!("Error generating code for the body - {0}", e)
        };
        LLVMBuildRet(ctxt.builder, value_ref);
        ctxt.sym_tab.pop();
        ctxt.proto_map.insert("not", true);
    }
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
              let (arg_type, arg_expr) = (&lst[0].0, &lst[0].1);
              debug_assert!(*arg_type == TType::TString || *arg_type == TType::TInt32,
                            format!("Arg type of print is {0}", arg_type));

              let print_function : LLVMValueRef;
              //check if we already have a prototype defined
              if !ctxt.proto_map.contains_key("printf"){
                  let print_ty = LLVMIntTypeInContext(ctxt.context, 32);
                  let mut pf_type_args_vec = vec![LLVMPointerType(LLVMIntTypeInContext(ctxt.context, 8), 0)];
                  let proto = LLVMFunctionType(print_ty, pf_type_args_vec.as_mut_ptr(), 1, 1);
                  print_function = LLVMAddFunction(ctxt.module,
                                                   c_str_ptr!("printf"),
                                                   proto);
                  ctxt.proto_map.insert("printf", true);
              }
              else{
                  print_function = LLVMGetNamedFunction(ctxt.module, c_str_ptr!("printf"));
              }

              let mut pf_args = Vec::new();
              let mut args_count = 1;
              if *arg_type == TType::TString {
                  let gstr = arg_expr.codegen(ctxt);
                  pf_args.push(gstr.unwrap());
              }

              if *arg_type == TType::TInt32{
                  args_count = 2;
                  let gstr = LLVMBuildGlobalStringPtr(ctxt.builder, 
                                                      c_str_ptr!("%d\n"), 
                                                      c_str_ptr!(".str"));
                  pf_args.push(gstr);
                  let l = match &arg_expr.codegen(ctxt){
                      &Ok(val) => val,
                      &Err(ref err) => panic!("Error occurred")
                  };
                  pf_args.push(l);
              }

              Some(LLVMBuildCall(ctxt.builder,
                                 print_function,
                                 pf_args.as_mut_ptr(),
                                 args_count,
                                 c_str_ptr!("call")))
          },
          "size" => {
              debug_assert!(args.is_some(), "No args passed to size()");
              let lst = args.as_ref().unwrap();
              debug_assert!(lst.len() == 1, "One arg should be passed to size()");
              let (arg_type, arg_expr) = (&lst[0].0, &lst[0].1);
              debug_assert!(*arg_type == TType::TString, format!("Arg type of print is {0}", arg_type));

              let size_function : LLVMValueRef;
              //check if we already have a prototype defined
              if !ctxt.proto_map.contains_key("size"){
                  let size_ty = LLVMIntTypeInContext(ctxt.context, 32);
                  let mut size_type_args_vec = vec![LLVMPointerType(LLVMIntTypeInContext(ctxt.context, 8), 0)];
                  let proto = LLVMFunctionType(size_ty, size_type_args_vec.as_mut_ptr(), 1, 0);
                  size_function = LLVMAddFunction(ctxt.module,
                                                   c_str_ptr!("strlen"),
                                                   proto);
                  ctxt.proto_map.insert("size", true);
              }
              else{
                  size_function = LLVMGetNamedFunction(ctxt.module, c_str_ptr!("printf")); 
              }

              let mut size_args = Vec::new();
              let mut args_count = 1;
              let gstr = arg_expr.codegen(ctxt);
              size_args.push(gstr.unwrap());

              Some(LLVMBuildCall(ctxt.builder,
                                 size_function,
                                 size_args.as_mut_ptr(),
                                 args_count,
                                 c_str_ptr!("call")))
          },
          "not" => {
              debug_assert!(args.is_some(), "No args passed to not()");
              let lst = args.as_ref().unwrap();
              debug_assert!(lst.len() == 1, "One arg should be passed to not()");
              let (arg_type, arg_expr) = (&lst[0].0, &lst[0].1);
              debug_assert!(*arg_type == TType::TString || *arg_type == TType::TInt32);

              let not_function =  LLVMGetNamedFunction(ctxt.module, c_str_ptr!("not"));
              let mut not_args= Vec::new();
              let mut args_count = 1;
              let l = match &arg_expr.codegen(ctxt){
                  &Ok(val) => val,
                  &Err(ref err) => panic!("Error occurred - {0}", err)
              };
              not_args.push(l);

              Some(LLVMBuildCall(ctxt.builder,
                                 not_function,
                                 not_args.as_mut_ptr(),
                                 args_count,
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
            &TType::TString => LLVMPointerType(LLVMIntTypeInContext(ctxt.context, 8), 0),
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

        macro_rules! build_relational_instrs{
            ($fun : ident, $pred : path, $e1:ident, $e2:ident, $s : expr) => {{
                let ev1 = try!($e1.codegen(ctxt));
                let ev2 = try!($e2.codegen(ctxt));
                Ok($fun(ctxt.builder, $pred, ev1, ev2, $s.as_ptr() as *const i8))
            }}
        }
        unsafe{
            match self{
                &Expr::NumExpr(ref i) => {
                    let ty = LLVMIntTypeInContext(ctxt.context, 32);
                    Ok(LLVMConstInt(ty, *i as u64, 0))
                },
                &Expr::StringExpr(ref s) => {
                    Ok(LLVMBuildGlobalStringPtr(ctxt.builder, 
                                             c_str_ptr!(&*(s.clone())),
                                             c_str_ptr!(".str")))
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
                &Expr::EqualsExpr(ref e1, ref e2) => {
                    build_relational_instrs!(LLVMBuildICmp, llvm::LLVMIntPredicate::LLVMIntEQ, e1, e2, "eqcmp_tmp")
                },
                &Expr::LessThanExpr(ref e1, ref e2) => {
                    build_relational_instrs!(LLVMBuildICmp, llvm::LLVMIntPredicate::LLVMIntSLT, e1, e2, "lecmp_tmp")
                },
                &Expr::GreaterThanExpr(ref e1, ref e2) => {
                    build_relational_instrs!(LLVMBuildICmp, llvm::LLVMIntPredicate::LLVMIntSGT, e1, e2, "gtcmp_tmp")
                },
                &Expr::NotEqualsExpr(ref e1, ref e2) => {
                    build_relational_instrs!(LLVMBuildICmp, llvm::LLVMIntPredicate::LLVMIntNE, e1, e2, "necmp_tmp")
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
                        Ok(LLVMBuildLoad(ctxt.builder, _optional.as_ref().unwrap().alloca_ref(), c_str_ptr!(&*id.clone())))
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
                    LLVMBuildStore(ctxt.builder, from_code, from_var);

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
                                for &(ref ty, ref e) in optional_args.as_ref().unwrap(){
                                    let c = try!(e.codegen(ctxt));
                                    pf_args.push(c);
                                }
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
                                            pf_args.len() as u32,
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
                                let mut type_args = Vec::new();
                                let optional_params = params.as_ref();
                                //FIXME simplify this param checking condition
                                if optional_params.is_some() && optional_params.unwrap().len() > 0{
                                    for p in optional_params.unwrap(){
                                        let param_llvm_type = get_llvm_type_for_ttype(&p.1, ctxt);
                                        type_args.push(param_llvm_type); 
                                    }
                                }
                                let proto = LLVMFunctionType(llvm_ty, 
                                                             type_args.as_mut_ptr(),
                                                             if optional_params.is_some(){
                                                                 optional_params.unwrap().len() as u32
                                                             }
                                                             else{
                                                                 0
                                                             }, 
                                                             0);
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
                                
                                ctxt.sym_tab.push((String::from("<marker>"),
                                                   None));
                                //build allocas for params
                                if optional_params.is_some() && optional_params.unwrap().len() > 0{
                                    let c = LLVMCountParams(function) as usize;
                                    let mut params_vec = Vec::with_capacity(c);
                                    let p = params_vec.as_mut_ptr();
                                    mem::forget(params_vec);
                                    LLVMGetParams(function, p);
                                    let mut v = Vec::from_raw_parts(p, c, c);
                                    //assert_eq!(params_vec.len(), 1);
                                    for (value_ref, param) in v.iter().zip(optional_params.unwrap()){
                                        let alloca = LLVMBuildAlloca(ctxt.builder,
                                                                     get_llvm_type_for_ttype(&param.1, ctxt),
                                                                     c_str_ptr!(&*param.0));
                                        LLVMBuildStore(ctxt.builder,
                                                       *value_ref,
                                                       alloca);
                                        ctxt.sym_tab.push((param.0.clone(), 
                                                           Some(Box::new(Var::new(param.0.clone(), param.1.clone(), alloca)))));

                                    }
                                }
                                let value_ref = try!(body.codegen(ctxt));
                                if *ty == TType::TVoid{
                                    LLVMBuildRetVoid(ctxt.builder);
                                }
                                else{
                                    LLVMBuildRet(ctxt.builder, value_ref);
                                }

                                //pop all local symbols belonging to the current function
                                while !ctxt.sym_tab.last().unwrap().1.is_none(){
                                    ctxt.sym_tab.pop();
                                }
                                ctxt.sym_tab.pop(); 
                            }, 
                            &Decl::VarDec(ref name, ref ty, ref rhs) => {
                                let llvm_ty = get_llvm_type_for_ttype(ty, ctxt);
                                let alloca = LLVMBuildAlloca(ctxt.builder, llvm_ty, c_str_ptr!(&(*name.clone())));
                                let rhs_value_ref = try!(rhs.codegen(ctxt));
                                LLVMBuildStore(ctxt.builder,
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

trait StdFunctionCodeBuilder{
    fn std_fn_codegen(&self, ctxt : &mut Context);
}

impl StdFunctionCodeBuilder for Expr{
    fn std_fn_codegen(&self, ctxt : &mut Context){
        match *self{
            Expr::NumExpr(_) |
            Expr::StringExpr(_) |
            Expr::IdExpr(_) => return,
            Expr::AddExpr(ref e1, ref e2) |
            Expr::SubExpr(ref e1, ref e2) |
            Expr::MulExpr(ref e1, ref e2) |
            Expr::DivExpr(ref e1, ref e2) |
            Expr::LessThanExpr(ref e1, ref e2) |
            Expr::GreaterThanExpr(ref e1, ref e2) |
            Expr::EqualsExpr(ref e1, ref e2) => {
                e1.std_fn_codegen(ctxt);
                e2.std_fn_codegen(ctxt);
            },
            Expr::IfThenElseExpr(ref cond_expr, ref then_expr, ref else_expr) => {
                cond_expr.std_fn_codegen(ctxt);
                then_expr.std_fn_codegen(ctxt);
                else_expr.std_fn_codegen(ctxt); 
            },
            Expr::ForExpr(_, ref from, ref to, ref do_expr) => {
                from.std_fn_codegen(ctxt);
                to.std_fn_codegen(ctxt);
                do_expr.std_fn_codegen(ctxt);
            },
            Expr::LetExpr(ref decls, ref body) =>{
                for decl in &*decls {
                    match decl {
                        &Decl::FunDec(_, _, _, ref body, _) => {
                            body.std_fn_codegen(ctxt);
                        },
                        &Decl::VarDec(_, _, ref rhs) => {
                            rhs.std_fn_codegen(ctxt);
                        }
                        _ => {}
                    }

                    body.as_ref().unwrap().std_fn_codegen(ctxt);
                }
            }
            Expr::CallExpr(ref id, ref optional_ty_expr_args) => {
                match &**id{
                    "not" => {
                        if !ctxt.proto_map.contains_key("not"){
                            unsafe{ 
                                let not_function : LLVMValueRef;
                                //check if we already have a prototype defined
                                let not_ty = LLVMIntTypeInContext(ctxt.context, 32);
                                let mut not_type_args_vec = vec![LLVMIntTypeInContext(ctxt.context, 32)];
                                let proto = LLVMFunctionType(not_ty, not_type_args_vec.as_mut_ptr(), 1, 0);
                                not_function = LLVMAddFunction(ctxt.module,
                                                               c_str_ptr!("not"),
                                                               proto);
                                let bb = LLVMAppendBasicBlockInContext(ctxt.context,
                                                                       not_function,
                                                                       c_str_ptr!("entry"));

                                let func = Function::new(String::from("not"), not_function);
                                //FIXME this should be inserted at the beginning to indicate the fact that
                                //it belongs to the global scope
                                ctxt.sym_tab.push((String::from("not"), Some(Box::new(func))));
                                LLVMPositionBuilderAtEnd(ctxt.builder, bb);

                                //build allocas for params
                                let c = LLVMCountParams(not_function) as usize;
                                assert_eq!(c, 1);
                                let mut params_vec = Vec::with_capacity(c);
                                let p = params_vec.as_mut_ptr();
                                mem::forget(params_vec);
                                LLVMGetParams(not_function, p);
                                let mut v = Vec::from_raw_parts(p, c, c);
                                assert_eq!(v.len(), 1);
                                //assert_eq!(params_vec.len(), 1);
                                let alloca = LLVMBuildAlloca(ctxt.builder,
                                                             LLVMIntTypeInContext(ctxt.context, 32),
                                                             c_str_ptr!("a"));
                                LLVMBuildStore(ctxt.builder,
                                               v[0],
                                               alloca);
                                ctxt.sym_tab.push((String::from("a"), 
                                                   Some(Box::new(Var::new(String::from("a"), TType::TInt32, alloca)))));
                                let body = IfThenElseExpr(B(EqualsExpr(B(IdExpr(String::from("a"))), B(NumExpr(0)))),
                                B(NumExpr(1)),
                                B(NumExpr(0)));
                                let value_ref = match body.codegen(ctxt){
                                    Ok(v_ref) => v_ref,
                                    Err(e) => panic!("Error generating code for the body - {0}", e)
                                };
                                LLVMBuildRet(ctxt.builder, value_ref);
                                ctxt.sym_tab.pop();
                                ctxt.proto_map.insert("not", true);
                            }
                        }
                    },
                    _ => {}
                }
                
                if optional_ty_expr_args.is_some(){
                    for &(_ , ref e) in optional_ty_expr_args.as_ref().unwrap(){
                        e.std_fn_codegen(ctxt);
                    }
                }
            },
            _ => {panic!("Expression not covered yet for intrinsic code generation")}
        }
    }
}
pub fn translate(expr : &Expr) -> Option<Context>{
    let mut ctxt = Context::new("main_mod");
    unsafe{
        let r = LLVM_InitializeNativeTarget();
        assert_eq!(r, 0);
        LLVM_InitializeNativeAsmPrinter();

        expr.std_fn_codegen(&mut ctxt);

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
fn test_prsr_bcknd_intgrtion_prnt_call() {
    let mut p = Parser::new("print(\"Grrrr!\n\")".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
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
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
}
#[test]
fn test_prsr_bcknd_intgrtion_let_blk() {
    let mut p = Parser::new("let function foo() = print(\"Grrrr!\n\") in foo() end".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_if_then_expr() {
    let mut p = Parser::new("let function foo()  = if 0 then print(\"rust\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_if_then_expr_with_div_expr() {
    let mut p = Parser::new("let function foo()  = if 1/1 then print(\"rust\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_if_then_expr_with_mul_expr() {
    let mut p = Parser::new("let function foo()  = if 1*1 then print(\"ruby\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_if_then_expr_with_less_than_expr() {
    let mut p = Parser::new("let function foo() = if 1<1 then print(\"ruby\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
#[should_panic(expected="Both types of a relational operator must match and be of type int or string.")]
fn test_prsr_bcknd_intgrtion_less_than_expr_with_mismatched_types() {
    let mut p = Parser::new("let function foo() = if 1< \"abhi\" then print(\"ruby\n\") else print(\"c++\n\") in foo() end".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}
#[test]
fn test_prsr_bcknd_intgrtion_var_decl() {
    let mut p = Parser::new("let var a : int :=1\n function foo()  = print(\"ruby\n\") in foo() end".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_for_loop() {
    let mut p = Parser::new("let function foo() = for i:=1 to 5 do print(\"ruby\n\") in foo() end".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}

#[test]
fn test_prsr_bcknd_intgrtion_print_num() {
    let mut p = Parser::new("print(1)".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.is_some(), true);
}
#[test]
#[should_panic(expected="Invalid call to 'foo'. Function not found.")]
fn test_prsr_bcknd_intgrtion_invalid_call() {
    let mut p = Parser::new("foo()".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
}


#[test]
#[should_panic(expected="Invalid reference to variable 'i'")]
fn test_prsr_bcknd_intgrtion_invalid_reference_to_var() {
    let mut p = Parser::new("let var a : int :=i in foo()".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
}

#[test]
fn test_prsr_bcknd_intgrtion_var_assignment_to_var() {
    let mut p = Parser::new("let var i : int := 1\nvar a : int :=i in print(\"\")".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.unwrap().sym_tab.len(), 2);
}

#[test]
#[should_panic(expected="Invalid reference to variable 'foo'. Different binding found.")]
fn test_prsr_bcknd_intgrtion_invalid_reference_to_var_defined_as_function() {
    let mut p = Parser::new("let function foo() = print(\"b\")\nvar i : int := foo\n in print(\"\")".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
}

#[test]
#[should_panic(expected="Invalid reference to function 'foo'. Different binding found.")]
fn test_prsr_bcknd_intgrtion_invalid_reference_to_func_defined_as_var() {
    let mut p = Parser::new("let var foo : int := 1\n in foo()".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
}

//#[test]
fn test_prsr_bcknd_intgrtion_empty_sym_tab_after_function_scope_ends() {
    let mut p = Parser::new("let var a : int := 1\nfunction foo(a:int, b:int) = print(\"abhi\")\n in foo()".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    assert_eq!(ctxt.unwrap().sym_tab.len(), 2);
}

#[test]
fn test_prsr_bcknd_intgrtion_function_with_2_int_params_with_a_call() {
    let mut p = Parser::new("let function add(a:int, b:int) : int = a+b\n in add(1, 2)".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&*b_expr);
    //assert_eq!(ctxt.unwrap().sym_tab.len(), 1);
}

#[test]
fn test_prsr_bcknd_intgrtion_print_addition_call_result() {
    let mut p = Parser::new("let function add(a:int, b:int) : int = a+b\n in print(add(1,2))".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&mut *b_expr);
    //assert_eq!(ctxt.unwrap().sym_tab.len(), 1);
}

#[test]
fn test_prsr_bcknd_intgrtion_print_string_return_call_result() {
    let mut p = Parser::new("let function add() : string = \"abhi\n\"\n in print(add())".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&mut *b_expr);
    //assert_eq!(ctxt.unwrap().sym_tab.len(), 1);
}

#[test]
fn test_prsr_bcknd_intgrtion_print_not_return_call_result() {
    let mut p = Parser::new("print(not(0))".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&mut *b_expr);
    //assert_eq!(ctxt.unwrap().sym_tab.len(), 1);
}

#[test]
fn test_prsr_bcknd_intgrtion_print_size_return_call_result() {
    let mut p = Parser::new("print(size(\"abhi\"))".to_string());
    p.start_lexer();
    let mut tup = p.expr();
    let &mut (ref mut ty, ref mut b_expr) = tup.as_mut().unwrap();
    let mut v = TypeChecker::new();
    v.visit_expr(&mut *b_expr);
    let ctxt = translate(&mut *b_expr);
    link_object_code(ctxt.as_ref().unwrap());
    ctxt.unwrap().dump();
    //assert_eq!(ctxt.unwrap().sym_tab.len(), 1);
}
