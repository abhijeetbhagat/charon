use lua_types::*;
use block::*;
use trait_expression::*;
use trait_statement::*;
use expressions::*;

pub struct AssignStatement{
    line_pos : usize,
    lhs_sym : IdentExpression,
    rhs_expr : Box<Expression>
}

impl AssignStatement{
    pub fn new(line_pos : usize, lhs_sym : IdentExpression, rhs_expr : Box<Expression>)->Self{
        AssignStatement {line_pos : line_pos, lhs_sym : lhs_sym, rhs_expr : rhs_expr}
    }
}

impl Statement for AssignStatement{
    fn generate_code(&self)->Vec<String>{
        vec!["MOV 1,2".to_string()]
    }
}

pub struct FuncCallStatement{
    line_pos : usize,
    name : String
}
impl FuncCallStatement{
    pub fn new(line_pos : usize, name : String) -> Self{
        FuncCallStatement {line_pos : line_pos, name : name}
    }
}

impl Statement for FuncCallStatement{
    fn generate_code(&self)->Vec<String>{
        vec!["MOV 1,2".to_string()]
    }
}

pub struct LabelStatement{
    line_pos : usize,
    name : String
}

impl LabelStatement{
    pub fn new(line_pos : usize, name : String) -> Self{
        LabelStatement {line_pos : line_pos, name : name}
    }
}

impl Statement for LabelStatement{
    fn generate_code(&self) -> Vec<String>{
        let mut s = self.name.clone();
        s.push(':');
        vec![s]
    }
}

pub struct BreakStatement{
    line_pos : usize
}

impl BreakStatement{
    pub fn new(line_pos : usize) -> Self{
        BreakStatement {line_pos : line_pos} 
    }
}

impl Statement for BreakStatement{
    fn generate_code(&self) -> Vec<String>{
        //TODO: should this be bra instead?
        vec!["jmp".to_string()]
    }
}

pub struct GotoStatement{
    line_pos : usize,
    label : String
}

impl GotoStatement{
    pub fn new(line_pos : usize, label : String) -> Self{
        GotoStatement {line_pos : line_pos, label : label}
    }
}

impl Statement for GotoStatement{
    fn generate_code(&self) -> Vec<String>{
        vec![format!("bra {}", self.label)]
    }
}

pub struct DoStatement{
    line_pos : usize,
    pub block : Block
}

impl DoStatement{
    pub fn new(line_pos : usize)->Self{
        DoStatement {line_pos : line_pos, block : Block::new()}
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
    line_pos : usize,
    expr : Box<Expression>,
    do_stat : DoStatement
}

impl WhileStatement{
    fn new(line_pos : usize, expr : Box<Expression>)->Self{
        WhileStatement {line_pos : line_pos, expr : expr, do_stat : DoStatement::new(line_pos)}
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