use lexer::*;
use tokens::*;
use statements::*;
use expressions::*;
use lua_types::*;
use block::*;


struct Expression;

pub struct Parser{
    lexer : Lexer,
    block : Block
}

impl Parser{
    pub fn new(src : String)->Self{
        Parser {lexer : Lexer::new(src), block : Block::new()}
    }
    
    fn run(&mut self){
        //self.lexer.block();
        self.parse_block();
        self.block.generate();
    }
    
    fn parse_block(&mut self){
        self.lexer.get_char();
        self.stat();
    }
    
    fn stat(&mut self){
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
                                self.block.statements.push(Box::new(LabelStatement::new(self.lexer.curr_string.clone())))
                            },
                            _ => panic!("Expected '::'")
                        }
                    },
                    _ => panic!("Expected a label")
                }
            },
            Token::Break => {
                self.block.statements.push(Box::new(BreakStatement::new()))
            },
            Token::Goto => {
                match self.lexer.get_token(){
                    Token::Ident => {
                        self.block.statements.push(Box::new(GotoStatement::new(self.lexer.curr_string.clone())))
                    },
                    _ => panic!("Expected a label")
                }
            },
            Token::Do => {
                //make sure we track the closing end
                self.stat();
                if self.lexer.curr_token == Token::End{
                    return
                }
            },
            Token::Return => {},
            Token::Eof | Token::End => return,
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
    p.run();
    assert!(&*p.block.instructions[0] == "jmp");
}

#[test]
fn test_label(){
    let mut p = Parser::new("::abhi::".to_string());
    p.run();
    assert!(&*p.block.instructions[0] == "abhi:");
}

#[test]
fn test_break_and_label(){
    let mut p = Parser::new("break::abhi::".to_string());
    p.run();
    assert!(&*p.block.instructions[0] == "jmp");
    assert!(&*p.block.instructions[1] == "abhi:");
}

#[test]
fn test_goto(){
    let mut p = Parser::new("goto abhi".to_string());
    p.run();
    assert!(&*p.block.instructions[0] == "bra abhi");
}