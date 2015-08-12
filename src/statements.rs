use lua_types::*;
use block::*;
use trait_expression::*;
use trait_statement::*;
use expressions::*;

pub struct AssignStatement{
    lhs_sym : IdentExpression,
    rhs_expr : Box<Expression>
}

impl AssignStatement{
    pub fn new(lhs_sym : IdentExpression, rhs_expr : Box<Expression>)->Self{
        AssignStatement {lhs_sym : lhs_sym, rhs_expr : rhs_expr}
    }
}

impl Statement for AssignStatement{
    fn generate_code(&self)->Vec<String>{
        vec!["MOV 1,2".to_string()]
    }
}

pub struct FuncCallStatement{
    name : String
}
impl FuncCallStatement{
    pub fn new(name : String) -> Self{
        FuncCallStatement {name : name}
    }
}

impl Statement for FuncCallStatement{
    fn generate_code(&self)->Vec<String>{
        vec!["MOV 1,2".to_string()]
    }
}

pub struct LabelStatement{
    name : String
}

impl LabelStatement{
    pub fn new(name : String) -> Self{
        LabelStatement {name : name}
    }
}

impl Statement for LabelStatement{
    fn generate_code(&self) -> Vec<String>{
        let mut s = self.name.clone();
        s.push(':');
        vec![s]
    }
}

pub struct BreakStatement;

impl BreakStatement{
    pub fn new() -> Self{
        BreakStatement
    }
}

impl Statement for BreakStatement{
    fn generate_code(&self) -> Vec<String>{
        //TODO: should this be bra instead?
        vec!["jmp".to_string()]
    }
}

pub struct GotoStatement{
    label : String
}

impl GotoStatement{
    pub fn new(label : String) -> Self{
        GotoStatement {label : label}
    }
}

impl Statement for GotoStatement{
    fn generate_code(&self) -> Vec<String>{
        vec![format!("bra {}", self.label)]
    }
}

pub struct DoStatement{
    pub block : Block
}

impl DoStatement{
    pub fn new()->Self{
        DoStatement {block : Block::new()}
    }
}

impl Statement for DoStatement{
    fn generate_code(&self) -> Vec<String>{
        let mut instructions : Vec<String> = Vec::new(); 
        for s in &self.block.statements{
            instructions.extend(s.generate_code().into_iter());
        }
        instructions
    }
}

pub struct WhileStatement{
    expr : Box<Expression>,
    do_stat : DoStatement
}

impl WhileStatement{
    fn new(expr : Box<Expression>)->Self{
        WhileStatement {expr : expr, do_stat : DoStatement::new()}
    }
}

impl Statement for WhileStatement{
    fn generate_code(&self) -> Vec<String>{
        let mut instructions : Vec<String> = Vec::new(); 
        for s in &self.do_stat.block.statements{
            instructions.extend(s.generate_code().into_iter());
        }
        instructions
    }
}

pub struct RepeatUntilStatement;