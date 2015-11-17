#![feature(rustc_private)]
extern crate llvm_sys as llvm;
extern crate libc;
use std::ptr;
use std::ffi;

use self::llvm::prelude::{LLVMContextRef, LLVMModuleRef, LLVMBuilderRef, LLVMValueRef, LLVMTypeRef};
use self::llvm::core::*;
use self::llvm::target::*;
use self::llvm::target_machine::*;
use helpers::*;
use std::process::Command;
use base::Context;

pub fn link(ctxt: &Context){
    unsafe {
        let target_ref = LLVMGetFirstTarget();
        let target_mc = LLVMCreateTargetMachine(target_ref, 
                                                LLVMGetDefaultTargetTriple(),
                                                c_str_ptr!("i386"),
                                                c_str_ptr!(""),
                                                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                                                LLVMRelocMode::LLVMRelocDefault,
                                                LLVMCodeModel::LLVMCodeModelDefault );
        assert!(target_mc != ptr::null_mut());
        LLVMTargetMachineEmitToFile(target_mc, 
                                    ctxt.module,
                                    c_str_mut_ptr!("tmp.o"),
                                    LLVMCodeGenFileType::LLVMObjectFile,
                                    c_str_mut_ptr!("") as *mut *mut libc::c_char);

        let out = Command::new("ld")
            .arg("--dynamic-linker")
            .arg("/lib64/ld-linux-x86-64.so.2") 
            .arg("tmp.o")
            .arg("-o")
            .arg("first")
            .arg("-lc")
            .arg("--entry")
            .arg("main")
            .output()
            .unwrap_or_else(|e|{
                panic!("failed to compile - {}", e);
            });
        println!("{}", String::from_utf8_lossy(&out.stdout));
        println!("{}", String::from_utf8_lossy(&out.stderr));
    }
}
