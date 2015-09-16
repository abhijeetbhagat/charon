#![allow(dead_code)]

use std::mem;
use std::collections::{HashMap};
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
    seq_expr_list : Vec<B<Expr>>,
    last_expr_type : Option<TType>
}

impl Parser{
    pub fn new(src : String)->Self{
        Parser {
                lexer : Lexer::new(src),
                block_stack : BlockStack::new(),
                paren_stack : Vec::new(),
                seq_expr_list : Vec::new(),
                last_expr_type : None
        }

    }

    pub fn start_lexer(&mut self){
        self.lexer.get_char();
        self.lexer.get_token();
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
                let expr = Some(self.expr().unwrap().1);
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

    fn expr(&mut self) -> Option<(TType, B<Expr>)> {
        match self.lexer.curr_token{
            Token::Nil => {
                Some((TNil, B(NilExpr)))
            },
            Token::Number => {
                return self.parse_num_expr()
                //B(NumExpr(self.lexer.curr_string.clone().parse::<i32>().unwrap()))
            },
            Token::Ident => {
                return self.parse_ident_expr()
            },
            Token::Let =>{
                return self.parse_let_expr()
            },
            Token::Function => {
                return self.parse_function_decl();
            },
            Token::LeftParen => { //seqexpr
                self.paren_stack.push('(');

                while self.lexer.get_token() != Token::RightParen {
                    if self.lexer.curr_token == Token::SemiColon { continue; }
                    if self.lexer.curr_token == Token::Eof {break;}
                    let optional_expr = self.expr();
                    if optional_expr.is_some() {
                        let (ty, e) = optional_expr.unwrap();
                        self.seq_expr_list.push(e);
                        self.last_expr_type = Some(ty);
                    }

                    //check closing paren here because self.expr() above could have curr_token set to it
                    if self.lexer.curr_token == Token::RightParen{
                        break;
                    }
                }

                if self.lexer.curr_token == Token::Eof {
                    panic!("Unexpected eof encountered");
                }

                self.paren_stack.pop();
                if !self.paren_stack.is_empty() {
                    panic!("Missing ')'");
                }

                let last_type = mem::replace(&mut self.last_expr_type, None);
                let expr_list = mem::replace(&mut self.seq_expr_list, Vec::new());
                Some((last_type.unwrap(), B(SeqExpr(Some(expr_list)))))
            },
            // Token::RightParen => {
            //     if self.paren_stack.is_empty(){
            //         panic!("Mismatched parenthesis");
            //     }
            //     self.paren_stack.pop();
            //     //TODO mem::replace self.seq_expr_list with Vec::new and assign it to SeqExpr
            //     Some(B(SeqExpr(None)))
            // },
            _ => panic!("FIXME: handle more patterns")
        }
    }

    fn parse_let_expr(&mut self) -> Option<(TType, B<Expr>)>{
        let mut b = Block::new();
        //set parent-child relationship
        self.block_stack.push(b);
        let mut decls : Vec<Decl> = Vec::new();
        loop{
            match self.lexer.get_token() {
                Token::Type => { //typedec
                    self.parse_type_decl(&mut decls);
                },
                Token::Var => { //Vardec
                    self.parse_var_decl(&mut decls);
                },
                Token::Function => { //functiondec

                },

                //FIXME probably all these following guards are useless?
                Token::In => break,
                //FIXME Eof occurrence is an error
                Token::Eof => break,
                //FIXME End occurrence is an error
                Token::End => break,
                _ => panic!("Unexpected token. Expected a declaration or 'in'")
            }

            //this is needed because a var decl parse can set the curr_token to 'in'
            if self.lexer.curr_token == Token::In{
                break;
            }
        }//let loop ends

        if self.lexer.curr_token == Token::In{
            //FIXME get the list of exprs and the type of the last expr in the list
        }
        else{
            panic!("Expected 'in' after declarations");
        }
        return Some((TVoid, B(LetExpr(decls, None))))
    }

    fn parse_type_decl(&mut self, decls : &mut Vec<Decl>){
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
    }

    fn parse_var_decl(&mut self,  decls : &mut Vec<Decl>){
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
                                        let (ty, expr) = self.get_nxt_and_parse();
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
    }

    fn parse_ident_expr(&mut self) -> Option<(TType, B<Expr>)>{
        //check if symbol defined in the sym tab
        //if self.block_stack.last().unwrap().contains(self.lexer.curr_string)
        let op1 = B(IdExpr(self.lexer.curr_string.clone()));
        match self.lexer.get_token(){
            Token::LeftSquare => {}, //subscript
            Token::Dot => {}, //fieldexp
            Token::LeftParen => {}, //callexpr
            Token::Plus => {
                let (t, op2) = self.get_nxt_and_parse();

                //FIXME it's better to let the type-checker do the checking
                if t == TInt32{
                    return Some((TInt32, B(AddExpr(op1, op2))))
                }
                else{
                    panic!("Expected i32 as the type of rhs expression");
                }
            },
            _ => {
                //TVoid because we dont know the type of the identifier yet.
                return Some((TVoid, B(IdExpr(self.lexer.curr_string.clone()))))
            }
        }
        Some((TVoid, B(IdentExpr(self.lexer.curr_string.clone()))))
    }

    fn parse_num_expr(&mut self) -> Option<(TType, B<Expr>)>{
        let num = self.lexer.curr_string.parse::<i32>().unwrap();

        let op1 = B(NumExpr(num));
        match self.lexer.get_token(){
            Token::Plus => {
                let (t, op2) = self.get_nxt_and_parse();
                //FIXME it's better to use a type-checker
                if t == TInt32{
                    return Some((TInt32, B(AddExpr(op1, op2))))
                }
                else{
                    panic!("Expected i32 as the type of rhs expression");
                }
            },
            Token::Minus => {
                let (t, op2) = self.get_nxt_and_parse();
                //FIXME it's better to use a type-checker
                if t == TInt32{
                    return Some((TInt32, B(SubExpr(op1, op2))))
                }
                else{
                    panic!("Expected i32 as the type of rhs expression");
                }
            },
            //FIXME ';', ')' can be a encountered as well. deal with it.
            _ => {
                return Some((TInt32, op1))
            }
        }
    }

    fn parse_function_decl(&mut self) -> Option<(TType, B<Expr>)>{
        match self.lexer.get_token(){
            Token::Ident => {
                let id = self.lexer.curr_string.clone();

                //parse the parameters list
                let field_decs = self.parse_function_args_list();

                //parse return type
                let ret_type = self.parse_function_ret_type();
            },
            _ => panic!("Expected an id after 'function'")
        }
        //FIXME return the correct function return type and the whole body
        None
    }

    fn parse_function_args_list(&mut self) -> Option<HashMap<String, TType>> {
        match self.lexer.get_token(){
            Token::LeftParen => {
                let mut field_decs = HashMap::new();
                loop{
                    match self.lexer.get_token() {
                        Token::Comma => continue,
                        Token::RightParen => { //parameterless function
                            break;
                        },
                        Token::Eof => panic!("Unexpected eof encountered. Expected a ')' after field-declaration."),
                        Token::Ident => {
                            let id = self.lexer.curr_string.clone();
                            match  self.lexer.get_token() {
                                Token::Colon => {
                                    match self.lexer.get_token() {
                                        Token::Int |
                                        Token::TokString |
                                        Token::Ident => {
                                            let ty = Self::get_ty_from_string(self.lexer.curr_string.as_str());
                                            field_decs.insert(id, ty);
                                        },
                                        _ => panic!("Expected type-id after ':'")
                                    }
                                },
                                _ => panic!("Expected ':' after id")
                            }
                        },
                        _ => panic!("Expected a ')' or parameter id")
                    }
                }
                return if field_decs.is_empty() {None} else {Some(field_decs)}
            },
            _ => panic!("Expected a '(' after function id")
        }
    }

    fn parse_function_ret_type(&mut self) -> TType{
        match self.lexer.get_token() {
            Token::Colon => {
                match self.lexer.get_token() {
                    Token::Int |
                    Token::TokString |
                    Token::Ident => Self::get_ty_from_string(self.lexer.curr_string.as_str()),
                    _ => panic!("Expected a type after ':'")
                }
            }
            Token::Equals => {
                TVoid
            }
            _ => panic!("Expected ':' or '=' after the parameter list")
        }
    }

    fn get_ty_from_string(str_ : &str) -> TType{
        match str_ {
            "int" => TInt32,
            "string" => TString,
            _ => TCustom(str_.to_string())
        }
    }

    fn get_nxt_and_parse(&mut self) -> (TType, B<Expr>){
        self.lexer.get_token();
        self.expr().unwrap()
    }
}

#[test]
fn test_parse_func_ret_type_void(){
    let mut p = Parser::new(")=".to_string());
    p.start_lexer();
    let ty = p.parse_function_ret_type();
    assert_eq!(ty, TVoid);
}

#[test]
fn test_parse_func_ret_type_int(){
    let mut p = Parser::new(") : int =".to_string());
    p.start_lexer();
    let ty = p.parse_function_ret_type();
    assert_eq!(ty, TInt32);
}

#[test]
fn test_parse_func_ret_type_string(){
    let mut p = Parser::new(") : string =".to_string());
    p.start_lexer();
    let ty = p.parse_function_ret_type();
    assert_eq!(ty, TString);
}

#[test]
fn test_parse_func_ret_type_custom(){
    let mut p = Parser::new(") : custom =".to_string());
    p.start_lexer();
    let ty = p.parse_function_ret_type();
    assert_eq!(ty, TCustom("custom".to_string()));
}

#[test]
fn test_field_decs_none(){
    let mut p = Parser::new("f()".to_string());
    p.start_lexer();
    let m = p.parse_function_args_list();
    assert_eq!(m, None);
}

#[test]
fn test_field_decs_one_dec(){
    let mut p = Parser::new("f(a: int)".to_string());
    p.start_lexer();
    let m = p.parse_function_args_list();
    assert_eq!(m.is_some(), true);
    assert_eq!(m.unwrap().len(), 1);
}

#[test]
fn test_field_decs_two_decs(){
    let mut p = Parser::new("f(a: int, b:int)".to_string());
    p.start_lexer();
    let m = p.parse_function_args_list();
    assert_eq!(m.is_some(), true);
    assert_eq!(m.unwrap().len(), 2);
}

#[test]
fn test_field_decs_two_decs_int_string(){
    let mut p = Parser::new("f(a: int, b:string)".to_string());
    p.start_lexer();
    let m = p.parse_function_args_list().unwrap();
    assert_eq!(m.len(), 2);
    assert_eq!(m[&"a".to_string()], TType::TInt32);
    assert_eq!(m[&"b".to_string()], TType::TString);
}

#[test]
fn test_field_decs_one_dec_with_alias(){
    let mut p = Parser::new("f(a: myint)".to_string());
    p.start_lexer();
    let m = p.parse_function_args_list().unwrap();
    assert_eq!(m[&"a".to_string()], TType::TCustom("myint".to_string()));
}

#[test]
#[should_panic(expected="Unexpected eof encountered. Expected a ')' after field-declaration.")]
fn test_field_decs_no_closing_paren(){
    let mut p = Parser::new("f(a: myint".to_string());
    p.start_lexer();
    p.parse_function_args_list();
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

#[test]
fn test_1_seq_expr_able_to_parse() {
    let mut p = Parser::new("(1;)".to_string());
    p.start_lexer();
    assert_eq!(p.expr().is_some(), true);
}

#[test]
fn test_1_seq_expr_last_type_int() {
    let mut p = Parser::new("(1;)".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        SeqExpr(ref o) => {
            assert_eq!(o.as_ref().unwrap().len(), 1);
            match *o.as_ref().unwrap()[0]{
                NumExpr(ref n) => {
                    assert_eq!(*n, 1);
                },
                _ => {}
            }
        },
        _ => panic!("Invalid expr")
    }
}

#[test]
fn test_1_seq_expr_last_type_void() {
    let mut p = Parser::new("(a;)".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    assert_eq!(ty, TVoid);
}

#[test]
fn test_2_seq_exprs_last_type_void() {
    let mut p = Parser::new("(1;a;)".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    assert_eq!(ty, TVoid);
}

#[test]
fn test_2_seq_exprs_last_type_int() {
    let mut p = Parser::new("(a;1;)".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    assert_eq!(ty, TInt32);
}

#[test]
fn test_1_seq_expr_without_semicolon_type_int() {
    let mut p = Parser::new("(1)".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    assert_eq!(ty, TInt32);
}

#[test]
fn test_1_seq_expr_add_expr() {
    let mut p = Parser::new("(5+16)".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        SeqExpr(ref o) => {
            assert_eq!(o.as_ref().unwrap().len(), 1);
            match *o.as_ref().unwrap()[0]{
                AddExpr(ref e1, ref e2) => {
                    match **e1 {
                        NumExpr(ref n) => assert_eq!(*n, 5),
                        _ => {}
                    }
                    match **e2 {
                        NumExpr(ref n) => assert_eq!(*n, 16),
                        _ => {}
                    }
                },
                _ => {}
            }
        },
        _ => panic!("Invalid expr")
    }
}

#[test]
fn test_get_ty(){
    assert_eq!(Parser::get_ty_from_string("int"), TInt32);
    assert_eq!(Parser::get_ty_from_string("string"), TString);
    assert_eq!(Parser::get_ty_from_string("index_type"), TCustom("index_type".to_string()));
}
