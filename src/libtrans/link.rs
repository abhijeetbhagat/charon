extern crate llvm_sys as llvm;
extern crate libc;
use std::ptr;
use std::ffi;

use self::llvm::target_machine::*;
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

        let out = Command::new("gcc")
            .arg("tmp.o")
            .arg("-o")
            .arg("a.out")
            .output()
            .unwrap_or_else(|e|{
                panic!("failed to compile - {}", e);
            });
        println!("{}", String::from_utf8_lossy(&out.stdout));
        println!("{}", String::from_utf8_lossy(&out.stderr));
    }
}
