extern crate llvm_sys as llvm;
use self::llvm::prelude::{LLVMValueRef};
use syntax::ast::TType;

#[derive(Clone, Debug)]
pub enum SymbolKind{
    Variable,
    Function,
    Parameter,
    Type
}

pub trait Symbol{
    fn kind() -> SymbolKind where Self:Sized;
    fn id(&self) -> String; 
}

trait VarSymbol : Symbol{
    fn var_type(&self) -> &TType;
    fn alloca_ref(&self) ->  LLVMValueRef;
}

pub trait FunctionSymbol : Symbol{
    fn value_ref(&self) -> LLVMValueRef;
}

pub struct Var{
    id : String,
    var_type : TType,
    alloca_ref : LLVMValueRef
}

impl Var{
    pub fn new(id : String, ty : TType, alloca_ref : LLVMValueRef) -> Self{
        Var {
            id : id,
            var_type : ty,
            alloca_ref : alloca_ref
        } 
    }
}

impl Symbol for Var{
    fn id(&self) -> String{
        self.id.clone()
    }

    fn kind() -> SymbolKind{
        SymbolKind::Variable
    }
}
impl VarSymbol for Var{
   fn var_type(&self) -> &TType{
       &self.var_type
   }

   fn alloca_ref(&self) -> LLVMValueRef{
       self.alloca_ref
   }
}

pub struct Function{
    id : String,
    value_ref : LLVMValueRef
}

impl Function{
    pub fn new(id : String, value_ref : LLVMValueRef) -> Self{
        Function{
            id : id,
            value_ref : value_ref
        }
    }
}

impl Symbol for Function{
    fn id(&self) -> String{
        self.id.clone()
    }

    fn kind() -> SymbolKind{
        SymbolKind::Function
    }
}

impl FunctionSymbol for Function{
    fn value_ref(&self) -> LLVMValueRef{
        self.value_ref
    }
}
