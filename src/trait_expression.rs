use block::*;
pub trait Expression{
    fn semantic(&self, &Block);
}