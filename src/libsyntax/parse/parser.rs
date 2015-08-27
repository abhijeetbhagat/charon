#![allow(dead_code)]

use parse::lexer::*;
use parse::tokens::*;
use ast::{Stmt, Expr, Block, TType, Local, Decl};
use ast::Stmt::*;
use ast::Expr::*;
use ast::TType::*;
use ast::Decl::*;
//use ast::*;
use ptr::{B};
//use ast::{Expr, Stmt};

type BlockStack = Vec<Block>;

pub struct Parser{
    lexer : Lexer,
    block_stack : BlockStack,
    paren_stack : Vec<char>,
    seq_expr_list : Vec<B<Expr>>
}

impl Parser{
    pub fn new(src : String)->Self{
        Parser {lexer : Lexer::new(src),
                block_stack : BlockStack::new(),
                paren_stack : Vec::new(),
                seq_expr_list : Vec::new() }
    }

    pub fn run(& mut self)->Option<Block>{
        self.parse_block()
        //self.block.generate();
    }

    fn parse_block(& mut self)->Option<Block>{
        //let mut b = Block::new();
        //self.block_stack.push(b);
        self.lexer.get_char();
        self.program(); //begin parsing
        debug_assert!(self.block_stack.len() == 1, "Only parent block should be on
                                                    the stack when the parsing is finished");
        let mut main_block = self.block_stack.pop().unwrap();
        //main_block.generate();
        //if main_block.statements.len() == 0{
        if !main_block.expr.is_some(){
            None
        }
        else{
            Some(main_block)
        }
    }

    fn program(&mut self){
      loop{
        match self.lexer.get_token(){
            //FIXME semicolon handling should change:
            Token::SemiColon => continue,
            Token::Nil |
            Token::Number |
            Token::LeftParen |
            Token::Minus |
            Token::If |
            Token::While |
            Token::For |
            Token::Break |
            Token::Let |
            Token::Ident => {
                let expr = self.expr();
                self.block_stack.last_mut().unwrap().expr = expr;
                //FIXME should we break?
                break;
            },
            /*Token::Break => {
                let mut curr_block = self.block_stack.last_mut().unwrap();
                curr_block.statements.push(Self::mk_break_stmt());
            },*/
            /*Token::Do => {
                debug_assert!(self.block_stack.len() > 0, "No parent block on the stack");
                self.block_stack.push(Block::new());
                self.expr();
                if self.lexer.curr_token == Token::End{
                    //TODO make sure we track all block openings
                    let block = self.block_stack.pop().unwrap();
                    let mut curr_block = self.block_stack.last_mut().unwrap();
                    //curr_block.statements.push(Self::mk_block_stmt(block));
                }
            },*/
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

    // fn mk_var_decl(local : Local)->B<Stmt>{
    //     B(VarDeclStmt(local))
    // }
    //
    // fn mk_label_stmt(label : String)->B<Stmt>{
    //     B(ExprStmt(Self::mk_label_expr(label)))
    // }
    //
    // fn mk_label_expr(label: String)->B<Expr>{
    //     B(LabelExpr(label))
    // }
    //
    // fn mk_goto_stmt(label : String) -> B<Stmt>{
    //     B(ExprStmt(Self::mk_goto_expr(label)))
    // }
    //
    // fn mk_goto_expr(label : String) -> B<Expr>{
    //     B(GotoExpr(label))
    // }
    //
    // fn mk_break_stmt() -> B<Stmt>{
    //     B(ExprStmt(B(BreakExpr)))
    // }
    //
    // fn mk_block_stmt(block : Block) -> B<Stmt>{
    //     B(Stmt::ExprStmt(Self::mk_block_expr(block)))
    // }
    //
    // fn mk_block_expr(block : Block) -> B<Expr>{
    //     B(Expr::BlockExpr(B(block)))
    // }
    //
    // fn exprlist(&mut self){
    //      self.expr();
    //      match self.lexer.get_token(){
    //         Token::Ident => {},
    //         _ => {}
    //      }
    // }
    //
    // fn varlist(&mut self){
    //
    // }

    fn expr(&mut self) -> Option<B<Expr>> {
        match self.lexer.curr_token{
            Token::Nil => {Some(B(NilExpr))},
            Token::Number => {
                Some(B(NumExpr(self.lexer.curr_string.clone().parse::<i32>().unwrap())))
            },
            Token::Ident => {
                //check if symbol defined in the sym tab
                //if self.block_stack.last().unwrap().contains(self.lexer.curr_string)
                match self.lexer.get_token(){
                    Token::LeftSquare => {}, //subscript
                    Token::Dot => {}, //fieldexp
                    Token::LeftParen => {}, //callexpr

                    _ => {
                        return Some(B(IdExpr(self.lexer.curr_string.clone())))
                    }
                }
                Some(B(IdentExpr(self.lexer.curr_string.clone())))
            },
            Token::Let => {
                let mut b = Block::new();
                //set parent-child relationship
                self.block_stack.push(b);
                let mut decls = Vec::new();
                loop{
                    match self.lexer.get_token() {
                        Token::Type => { //typedec
                            match self.lexer.get_token() {
                                Token::Ident => {
                                    let id = self.lexer.curr_string.clone();
                                    match self.lexer.get_token(){
                                        Token::Equals => {
                                            match self.lexer.get_token(){
                                                Token::Int => decls.push(TyDec(id, TInt32)),
                                                Token::TokString => decls.push(TyDec(id, TString)),
                                                Token::Ident => decls.push(TyDec(id, TCustom(self.lexer.curr_string.clone()))),
                                                Token::Array => {
                                                    match self.lexer.get_token() {
                                                        Token::Of => {
                                                            match self.lexer.get_token() {
                                                                Token::Int => {},
                                                                Token::TokString => {},
                                                                Token::Ident => {},
                                                                _ => panic!("Expected either int, string or type-id")
                                                            }
                                                        },
                                                        _ => panic!("Expected 'of' after 'array'")
                                                    }
                                                },
                                                Token::LeftCurly => { //rectype

                                                },
                                                _ => panic!("Expected either int, string, type-id, array of, '{' after '='")
                                            }
                                        },
                                        _ => panic!("Expected '=' after type-id")
                                    }
                                },
                                _ => panic!("Expected identifier after 'type'")
                            }
                        },
                        Token::Var => { //Vardec
                            match self.lexer.get_token() {
                                Token::Ident => {
                                    let id = self.lexer.curr_string.clone();
                                    match self.lexer.get_token() {
                                        Token::Colon => {
                                            match self.lexer.get_token() {
                                                Token::Int => {
                                                    match self.lexer.get_token(){
                                                        Token::ColonEquals => {
                                                            //get rhs expr and its type
                                                            let (ty, expr) = self.evaluable_expr();
                                                            self.block_stack.last_mut().unwrap().sym_tab.borrow_mut().insert(id.clone(), ty);
                                                            decls.push(VarDec(id.clone(), TInt32, expr));
                                                        },
                                                        _ => panic!("Expected ':='")
                                                    }
                                                },
                                                Token::TokString => {
                                                    match self.lexer.get_token(){
                                                        Token::ColonEquals => {
                                                            self.expr();
                                                        },
                                                        _ => panic!("Expected ':='")
                                                    }
                                                },
                                                _ => panic!("expr : pattern not covered")
                                            }
                                        },
                                        _ => panic!("Expected ':' after identifier")
                                    }
                                },
                                _ => panic!("Expected an identifier")
                            }
                        },
                        Token::Function => { //functiondec

                        },
                        Token::In => break,
                        //FIXME Eof occurrence is an error
                        Token::Eof => break,
                        //FIXME End occurrence is an error
                        Token::End => break,
                        _ => panic!("Unexpected token. Expected a declaration or 'in'")
                    }
                }//let loop ends
                //FIXME start scanning expressions after 'in'
                return Some(B(LetExpr(decls, None)))
            },
            Token::LeftParen => { //seqexpr
                self.paren_stack.push('(');
                let e = self.expr();
                if e.is_some() {
                    self.seq_expr_list.push(e.unwrap());
                }
                //FIXME remove this:
                Some(B(SeqExpr(None)))
            },
            Token::RightParen => {
                if self.paren_stack.is_empty(){
                    panic!("Mismatched parenthesis");
                }
                self.paren_stack.pop();
                //TODO mem::replace self.seq_expr_list with Vec::new and assign it to SeqExpr
                Some(B(SeqExpr(None)))
            },
            _ => Some(B(IdentExpr("fsf".to_string())))
        }
    }

    fn evaluable_expr(&mut self)->(TType, B<Expr>){
        match self.lexer.get_token() {
            Token::Ident => {
                let id = self.lexer.curr_string.clone();
                match self.lexer.get_token() {
                    Token::Dot => {
                        match self.lexer.get_token() {
                            Token::Ident => {

                            },
                            _ => panic!("evaluable_expr : pattern not covered")
                        }
                    },
                    Token::LeftSquare => {

                    },
                    //FIXME can new line be replaced with a semicolon as a decl terminator instead?
                    Token::NewLine => {
                        //search the ident in the current symbol table
                        let mut b = self.block_stack.last_mut().unwrap();
                        if b.contains(&self.lexer.curr_string){
                            let m = b.sym_tab.borrow();
                            let ty = m.get(&self.lexer.curr_string).unwrap().clone();
                            return (ty.clone(), B(IdExpr(self.lexer.curr_string.clone())))
                        }
                        else{
                            panic!("Undefined symbol - {}", self.lexer.curr_string);
                        }
                    }
                    _ => panic!("evaluable_expr : pattern not covered")
                }
            },
            Token::Number => {
                //FIXME parse the whole numeric expression
                let num = self.lexer.curr_string.parse::<i32>().unwrap();
                let op1 = B(NumExpr(num));
                match self.lexer.get_token(){
                    Token::Plus => {
                        let (t, op2) = self.evaluable_expr();
                        //FIXME it's better to use a type-checker
                        if t == TInt32{
                            return (TInt32, B(AddExpr(op1, op2)))
                        }
                        else{
                            panic!("Expected i32 as the type of rhs expression");
                        }
                    },
                    _ => return (TInt32, op1)
                }
                //return (TInt32, B(NumExpr(num)))
            },
            _ => panic!("evaluable_expr : pattern not covered")
        }
        //FIXME remove this:
        return (TInt32, B(NumExpr(1)))
    }
}

#[test]
fn test_let_var_decl_returns_block() {
    let mut p = Parser::new("let var a : int := 1 in end".to_string());
    assert_eq!(p.run().is_some(), true);
}

#[test]
fn test_let_var_decl_returns_let_expr() {
    let mut p = Parser::new("let var a : int := 1 in end".to_string());
    let b = p.run().unwrap();
    match *b.expr.unwrap(){
        LetExpr(ref v, ref o) => {
            assert_eq!(v.len(), 1);
            assert_eq!(o.is_some(), false);
            match v[0]{
                VarDec(ref id, ref ty, ref e) => {
                    assert_eq!(*id, "a".to_string());
                    match **e{ //**e means deref deref B<T> which results in T
                        NumExpr(ref n) => assert_eq!(1, *n),
                        _ => {}
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }
}

#[test]
fn test_let_var_decl_sym_tab_count() {
    let mut p = Parser::new("let var a : int := 1 in end".to_string());
    let b = p.run().unwrap();
    assert_eq!(b.sym_tab.borrow().len(), 1);
    assert_eq!(b.sym_tab.borrow().get(&"a".to_string()), Some(&TInt32));
}

#[test]
fn test_let_add_expr() {
    let mut p = Parser::new("let var a : int := 1 + 3 + 1 in end".to_string());
    let b = p.run().unwrap();
    match *b.expr.unwrap(){
        LetExpr(ref v, ref o) => {
            assert_eq!(v.len(), 1);
            assert_eq!(o.is_some(), false);
            match v[0]{
                VarDec(ref id, ref ty, ref e) => {
                    assert_eq!(*id, "a".to_string());
                    match **e{ //**e means deref deref B<T> which results in T
                        AddExpr(ref e1, ref e2) => {
                            match **e1{
                                NumExpr(ref n) => assert_eq!(*n, 1),
                                _ => panic!("num expr expected")
                            }

                            match **e2{
                                AddExpr(ref e1, ref e2) => {
                                    match **e1{
                                        NumExpr(ref n) => assert_eq!(*n, 3),
                                        _ => panic!("num expr expected")
                                    }

                                    match **e2{
                                        NumExpr(ref n) => assert_eq!(*n, 1),
                                        _ => panic!("num expr expected")
                                    }
                                },
                                _ => panic!("add expr expected")
                            }
                        },
                        _ => panic!("add expr expected")
                    }
                },
                _ => panic!("ver decl expected")
            }
        },
        _ => panic!("let expr expected")
    }
}

#[test]
fn test_parse_2_vars_in_let() {
    let mut p = Parser::new("let var a : int := 1\nvar b : int:=2\n in end".to_string());
    let b = p.run().unwrap();
    match *b.expr.unwrap(){
        LetExpr(ref v, ref o) => {
            assert_eq!(v.len(), 2);
        },
        _ => {}
    }
}
