use lexer::*;
use tokens::*;
use statements::*;
use expressions::*;
use lua_types::*;
use block::*;


struct Expression;

type BlockStack = Vec<Block>;

pub struct Parser{
    lexer : Lexer,
    block_stack : BlockStack
}

impl<'a> Parser{
    pub fn new(src : String)->Self{
        Parser {lexer : Lexer::new(src),  block_stack : BlockStack::new()}
    }
    
    fn run(& mut self)->Block{
        self.parse_block()
        //self.block.generate();
    }
    
    fn parse_block(& mut self)->Block{
        let mut b = Block::new();
        self.block_stack.push(b);
        self.lexer.get_char();
        self.stat();
        let mut main_block = self.block_stack.pop().unwrap();
        main_block.generate();
        main_block
    }
    
    fn stat(&mut self){
      let mut block = self.block_stack.pop().unwrap();
      loop{
        match self.lexer.get_token(){
            Token::SemiColon => continue,
            Token::Ident => {
                //self.varlist();
                match self.lexer.get_token(){
                    Token::Assign => {
                       self.exprlist();
                    },
                    _ => panic!("Expected '='")
                }
            },
            Token::ColonColon => { 
                match self.lexer.get_token(){
                    Token::Ident => {
                        match self.lexer.get_token(){
                            Token::ColonColon => {
                                block.statements.push(Box::new(LabelStatement::new(self.lexer.curr_string.clone())))
                            },
                            _ => panic!("Expected '::'")
                        }
                    },
                    _ => panic!("Expected a label")
                }
            },
            Token::Break => {
                block.statements.push(Box::new(BreakStatement::new()))
            },
            Token::Goto => {
                match self.lexer.get_token(){
                    Token::Ident => {
                        block.statements.push(Box::new(GotoStatement::new(self.lexer.curr_string.clone())))
                    },
                    _ => panic!("Expected a label")
                }
            },
            Token::Do => {
                let mut do_stat = DoStatement::new();
                self.block_stack.push(Block::new());
                self.stat();
                if self.lexer.curr_token == Token::End{
                    //TODO make sure we track all block openings
                    do_stat.block = self.block_stack.pop().unwrap();
                    let mut parent_block = self.block_stack.pop().unwrap();
                    parent_block.statements.push(Box::new(do_stat));
                    self.block_stack.push(parent_block);
                    //self.block.statements.push(do_stat);                    
                    return
                }
            },
            Token::Return => {},
            Token::Eof => {self.block_stack.push(block); return},
            Token::End => {
                //TODO block stack pop
                return
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
    
    fn expr(&mut self){
        match self.lexer.get_token(){
            Token::Nil => {},
            Token::False => {},
            Token::True => {},
            Token::Number => {},
            Token::DotDotDot => {},
            Token::Function => {},
            Token::LeftCurly => {},
            _ => {}
        }
    }
}

#[test]
fn test_break(){
    let mut p = Parser::new("break".to_string());
    let b = p.run();
    assert!(&*b.instructions[0] == "jmp");
}

#[test]
fn test_label(){
    let mut p = Parser::new("::abhi::".to_string());
    let b = p.run();
    assert!(&*b.instructions[0] == "abhi:");
}

#[test]
fn test_break_and_label(){
    let mut p = Parser::new("break::abhi::".to_string());
    let b = p.run();
    assert!(&*b.instructions[0] == "jmp");
    assert!(&*b.instructions[1] == "abhi:");
}

#[test]
fn test_goto(){
    let mut p = Parser::new("goto abhi".to_string());
    let b = p.run();
    assert!(&*b.instructions[0] == "bra abhi");
}