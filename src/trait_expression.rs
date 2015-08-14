use block::*;
use visitor_expression::*;

pub trait Expression{
    fn semantic(&self, &Block);
    fn accept(&self, &SymbolVisitor);
}