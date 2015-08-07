use std::collections::HashMap;
use lua_types::*;
use traits::*;

pub struct Block{
    sym_tab : HashMap<String, LuaType>,
    pub statements : Vec<Box<Statement>>, //trait is boxed because it has no size known at compile-time. this is a trait object.
    pub instructions : Vec<String>
}

impl Block{
    pub fn new()->Self{
        Block {sym_tab : HashMap::new(), statements : Vec::new(), instructions : Vec::new()}
    }
    
    pub fn generate(&mut self){
        for s in &self.statements{
            /*for i in &s.generate_code(){
                println!("{}", i);
            }*/
            self.instructions.extend(s.generate_code().into_iter());
        }
    }
}