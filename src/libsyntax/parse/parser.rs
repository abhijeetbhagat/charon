#![allow(dead_code)]

use parse::lexer::*;
use parse::tokens::*;
use ast::{Stmt, Expr, Block, LuaType, Local};
use ast::Stmt::*;
use ast::Expr::*;
//use ast::*;
use ptr::{B};
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
                match self.lexer.get_token(){
                    Token::Equals => {
                       let expr = self.expr().unwrap();
                       let mut curr_block = self.block_stack.last_mut().unwrap();
                       let local = Local::new(self.lexer.curr_string.clone(), LuaType::LNil, expr);
                       curr_block.statements.push(Self::mk_var_decl(local));
                    },
                    _ => panic!("Expected '='")
                }
            },
            Token::Break => {
                let mut curr_block = self.block_stack.last_mut().unwrap();
                curr_block.statements.push(Self::mk_break_stmt());
            },
            Token::Do => {
                debug_assert!(self.block_stack.len() > 0, "No parent block on the stack");
                self.block_stack.push(Block::new());
                self.stat();
                if self.lexer.curr_token == Token::End{
                    //TODO make sure we track all block openings
                    let block = self.block_stack.pop().unwrap();
                    let mut curr_block = self.block_stack.last_mut().unwrap();
                    curr_block.statements.push(Self::mk_block_stmt(block));
                }
            },
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

    fn mk_var_decl(local : Local)->B<Stmt>{
        B(VarDeclStmt(local))
    }

    fn mk_label_stmt(label : String)->B<Stmt>{
        B(ExprStmt(Self::mk_label_expr(label)))
    }

    fn mk_label_expr(label: String)->B<Expr>{
        B(LabelExpr(label))
    }

    fn mk_goto_stmt(label : String) -> B<Stmt>{
        B(ExprStmt(Self::mk_goto_expr(label)))
    }

    fn mk_goto_expr(label : String) -> B<Expr>{
        B(GotoExpr(label))
    }

    fn mk_break_stmt() -> B<Stmt>{
        B(ExprStmt(B(BreakExpr)))
    }

    fn mk_block_stmt(block : Block) -> B<Stmt>{
        B(Stmt::ExprStmt(Self::mk_block_expr(block)))
    }

    fn mk_block_expr(block : Block) -> B<Expr>{
        B(Expr::BlockExpr(B(block)))
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

    fn expr(&mut self) -> Option<B<Expr>> {
        match self.lexer.get_token(){
            /*Token::Nil => {},
            Token::False => {},
            Token::True => {},*/
            Token::Number => {
                Some(B(NumExpr(self.lexer.curr_string.clone().parse::<i32>().unwrap())))
            },
            Token::Ident => {
                //check if symbol defined in the sym tab
                //if self.block_stack.last().unwrap().contains(self.lexer.curr_string)
                Some(B(IdentExpr(self.lexer.curr_string.clone())))
            }
            /*Token::DotDotDot => {},
            Token::Function => {},
            Token::LeftCurly => {},*/
            _ => {Some(B(IdentExpr("fsf".to_string())))}
        }
    }
}
