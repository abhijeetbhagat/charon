#![allow(dead_code)]

use parse::lexer::*;
use parse::tokens::*;
use ast::*;
//use ast::{Expr, Stmt};

type BlockStack = Vec<Block>;

pub struct Parser{
    lexer : Lexer,
    block_stack : BlockStack
}

impl Parser{
    pub fn new(src : String)->Self{
        Parser {lexer : Lexer::new(src),  block_stack : BlockStack::new()}
    }
    
    pub fn run(& mut self)->Option<Block>{
        self.parse_block()
        //self.block.generate();
    }
    
    fn parse_block(& mut self)->Option<Block>{
        let mut b = Block::new();
        self.block_stack.push(b);
        self.lexer.get_char();
        self.stat(); //begin parsing
        debug_assert!(self.block_stack.len() == 1, "Only parent block should be on the stack when the parsing is finished");
        let mut main_block = self.block_stack.pop().unwrap();
        main_block.generate();
        if main_block.statements.len() == 0{
            None
        }
        else{
            Some(main_block)
        }
    }
    
    fn stat(&mut self){
      //let mut block = self.block_stack.pop().unwrap();
      loop{
        match self.lexer.get_token(){
            Token::SemiColon => continue,
            Token::Ident => {
                //self.varlist();
                match self.lexer.get_token(){
                    Token::Assign => {
                       //let lhs = Expr::IdentExpr(self.lexer.curr_string.clone());//IdentExpr::new(self.lexer.curr_string.clone());
                       let expr = self.expr().unwrap();
                       let mut curr_block = self.block_stack.last_mut().unwrap();
                       let local = Local::new(self.lexer.curr_string.clone(), LuaType::LNil, expr);
                       curr_block.statements.push(Box::new(Stmt::VarDeclStmt(local)));//(Box::new(AssignStatement::new(self.lexer.line_pos, lhs, expression)));
                       
                    },
                    _ => panic!("Expected '='")
                }
            },
            Token::ColonColon => { 
                match self.lexer.get_token(){
                    Token::Ident => {
                        match self.lexer.get_token(){
                            Token::ColonColon => {
                                //add statement to the current block scope
                                let mut curr_block = self.block_stack.last_mut().unwrap();
                                curr_block.statements.push(Box::new(Stmt::ExprStmt(Box::new(Expr::LabelExpr(self.lexer.curr_string.clone())))));//(Box::new(LabelStatement::new(self.lexer.line_pos, self.lexer.curr_string.clone())))
                            },
                            _ => panic!("Expected '::'")
                        }
                    },
                    _ => panic!("Expected a label")
                }
            },
            Token::Break => {
                let mut curr_block = self.block_stack.last_mut().unwrap();
                curr_block.statements.push(Box::new(Stmt::ExprStmt(Box::new(Expr::BreakExpr))));//BreakStatement::new(self.lexer.line_pos)))
            },
            Token::Goto => {
                match self.lexer.get_token(){
                    Token::Ident => {
                        let mut curr_block = self.block_stack.last_mut().unwrap();
                        curr_block.statements.push(Box::new(Stmt::ExprStmt(Box::new(Expr::GotoExpr(self.lexer.curr_string.clone())))));//GotoStatement::new(self.lexer.line_pos, self.lexer.curr_string.clone())))
                    },
                    _ => panic!("Expected a label")
                }
            },
            Token::Do => {
                let mut do_stat = DoStatement::new(self.lexer.line_pos);
                debug_assert!(self.block_stack.len() > 0, "No parent block on the stack");
                self.block_stack.push(Block::new());
                self.stat();
                if self.lexer.curr_token == Token::End{
                    //TODO make sure we track all block openings
                    //do_stat.block
                    let block = self.block_stack.pop().unwrap();
                    let mut curr_block = self.block_stack.last_mut().unwrap();
                    curr_block.statements.push(Box::new(Stmt::ExprStmt(Box::new(Expr::BlockExpr(Box::new(block))))));//do_stat));
                }
            },
            Token::Return => {},
            Token::Eof => {return},
            Token::End => {
                //TODO block stack pop
                return
                //continue;
            },
            _ => {panic!("Invalid token");}
        }
      }
    }
    
    fn exprlist(&mut self){
         self.expr();
         match self.lexer.get_token(){
            Token::Ident => {},
            _ => {}
         }
    }
    
    fn varlist(&mut self){
            
    }
    
    fn expr(&mut self) -> Option<Box<Expr>> {
        match self.lexer.get_token(){
            /*Token::Nil => {},
            Token::False => {},
            Token::True => {},*/
            Token::Number => {
                Some(Box::new(Expr::NumExpr(self.lexer.curr_string.clone().parse::<i32>().unwrap())))
            },
            Token::Ident => {
                //check if symbol defined in the sym tab
                //if self.block_stack.last().unwrap().contains(self.lexer.curr_string)
                Some(Box::new(Expr::IdentExpr(self.lexer.curr_string.clone()))) 
            }
            /*Token::DotDotDot => {},
            Token::Function => {},
            Token::LeftCurly => {},*/
            _ => {Some(Box::new(Expr::IdentExpr("fsf".to_string())))}
        }
    }
}

#[test]
fn test_break(){
    let mut p = Parser::new("break".to_string());
    let b = p.run().unwrap();
    assert!(&*b.instructions[0] == "jmp");
}

#[test]
fn test_label(){
    let mut p = Parser::new("::abhi::".to_string());
    let b = p.run().unwrap();
    assert!(&*b.instructions[0] == "abhi:");
}

#[test]
fn test_break_and_label(){
    let mut p = Parser::new("break::abhi::".to_string());
    let b = p.run().unwrap();
    assert!(&*b.instructions[0] == "jmp");
    assert!(&*b.instructions[1] == "abhi:");
}

#[test]
fn test_goto(){
    let mut p = Parser::new("goto abhi".to_string());
    let b = p.run().unwrap();
    assert!(&*b.instructions[0] == "bra abhi");
}

#[test]
fn test_nested_do_stmt(){
    let mut p = Parser::new("do do do break end break end break end".to_string());
    let b = p.run().unwrap();
    assert!(&*b.instructions[0] == "jmp");
    assert!(&*b.instructions[1] == "jmp");
    assert!(&*b.instructions[2] == "jmp");
}

#[test]
fn test_single_do_stmt(){
    let mut p = Parser::new("do  break end".to_string());
    let b = p.run().unwrap();
    assert!(&*b.instructions[0] == "jmp");
}

#[test]
fn test_single_assign_stmt(){
    let mut p = Parser::new("a=1".to_string());
    let b = p.run().unwrap();
    assert!(&*b.instructions[0] == "MOV 1,2");
}

#[test]
fn test_assign_break_in_do(){
    let mut p = Parser::new("do break a=1 end".to_string());
    let b = p.run().unwrap();
    assert!(&*b.instructions[0] == "jmp");
    assert!(&*b.instructions[1] == "MOV 1,2");
}
