use ast::TType;
#[derive(Clone, Debug)]
enum SymbolKind{
    Variable,
    Function,
    Parameter,
    Type
}
trait Symbol{
    fn kind() -> SymbolKind;
    fn id() -> String;
}

trait VarSymbol : Symbol{
    fn vartype() -> TType;
    fn alloca_ref() -> llvm::LLVMValueRef;
}

trait FunctionSymbol : Symbol{

}
