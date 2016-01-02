#![allow(dead_code)]

use std::mem;
use std::collections::{HashMap};
use parse::lexer::*;
use parse::tokens::*;
use ast::{Stmt, Expr, Block, TType, Local, Decl, OptionalTypeExprTupleList, OptionalParamInfoList};
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
            Token::Function |
            Token::Ident |
            Token::TokString => {
                let expr = Some(self.expr().unwrap().1);
                self.block_stack.last_mut().unwrap().expr = expr;
                //FIXME should we break?
                break;
            },
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

    //FIXME temporarily pub for integration testing
    pub fn expr(&mut self) -> Option<(TType, B<Expr>)> {
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
            Token::TokString => {
                return self.parse_string_expr()
            },
            Token::Let =>{
                return self.parse_let_expr()
            },
            // Token::Function => {
            //     return self.parse_function_decl()
            // },
            Token::LeftParen => { //seqexpr
                self.paren_stack.push('(');

                while self.lexer.get_token() != Token::RightParen {
                    //Be careful when you set value of self.lexer.curr_token between here and self.expr()
                    //since logic in expr() assumes that self.lexer.curr_token will already be set
                    if self.lexer.curr_token == Token::SemiColon { continue; }
                    if self.lexer.curr_token == Token::Eof { panic!("Unexpected eof encountered") }
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

                self.paren_stack.pop();
                if !self.paren_stack.is_empty() {
                    panic!("Missing ')'");
                }

                let last_type = mem::replace(&mut self.last_expr_type, None);
                let expr_list = mem::replace(&mut self.seq_expr_list, Vec::new());
                Some((last_type.unwrap(), B(SeqExpr(Some(expr_list)))))
            },
            Token::If => {
                return self.parse_if_then_else_expr()
            },
            Token::While => {
                return self.parse_while_expr()
            },
            Token::For => {
               return self.parse_for_expr()
            },
            // Token::RightParen => {
            //     if self.paren_stack.is_empty(){
            //         panic!("Mismatched parenthesis");
            //     }
            //     self.paren_stack.pop();
            //     //TODO mem::replace self.seq_expr_list with Vec::new and assign it to SeqExpr
            //     Some(B(SeqExpr(None)))
            // },
            Token::End => panic!("Unexpected 'end'. Expected an expr."),
            t =>{ println!("{:?}", t); panic!("FIXME: handle more patterns")}
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
                    self.parse_function_decl(&mut decls);
                },
                Token::NewLine => continue,
                //FIXME probably all these following guards are useless?
                Token::In => break,
                //FIXME Eof occurrence is an error
                Token::Eof => break,
                //FIXME End occurrence is an error
                Token::End => break,
                t => {println!("{:?}", t);panic!("Unexpected token. Expected a declaration or 'in'") }
            }

            //this is needed because a var decl parse can set the curr_token to 'in'
            if self.lexer.curr_token == Token::In{
                break;
            }
        }//let loop ends
        let (_ty, _expr) =
        if self.lexer.curr_token == Token::In{
            //FIXME get the list of exprs and the type of the last expr in the list
            self.lexer.get_token();
            let expr = self.expr();
            debug_assert!(expr.is_some(), "expr expected after 'in'");
            expr.unwrap()
        }
        else{
            panic!("Expected 'in' after declarations");
        };
        return Some((_ty, B(LetExpr(decls, Some(_expr)))))
    }

    fn parse_type_decl(&mut self, decls : &mut Vec<Decl>){
        match self.lexer.get_token() {
            Token::Ident => {
                let id = self.lexer.curr_string.clone();
                match self.lexer.get_token(){
                    Token::Equals => {
                        match self.lexer.get_token(){
                            Token::Int => decls.push(TypeDec(id, TInt32)),
                            Token::TokString => decls.push(TypeDec(id, TString)),
                            Token::Ident => decls.push(TypeDec(id, TCustom(self.lexer.curr_string.clone()))),
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
            Token::LeftParen => { //callexpr
                println!("parsing call");
                let args_list = self.parse_call_args();
                //FIXME should a marker type be used instead of TVoid to indicate that the type should be verified by the type-checker?
                match *op1 {
                    IdExpr(ref fn_name) => return Some((TVoid, B(CallExpr(fn_name.clone(), args_list)))),
                    _ => {}
                };
            },
            Token::Plus => {
                let (_, op2) = self.get_nxt_and_parse();

                return Some((TInt32, B(AddExpr(op1, op2))))

                //FIXME it's better to let the type-checker do the checking
                //if t == TInt32{
                    //return Some((TInt32, B(AddExpr(op1, op2))))
                //}
                //else{
                    //panic!("Expected i32 as the type of rhs expression");
                //}
            },
            Token::Equals => {
                let (_, op2) = self.get_nxt_and_parse();
                return Some((TVoid, B(EqualsExpr(op1, op2))))
            },
            _ => {
                //TVoid because we dont know the type of the identifier yet.
                return Some((TVoid, op1))
            }
        }
        Some((TVoid, op1)) 
    }

    fn parse_string_expr(&mut self) -> Option<(TType, B<Expr>)>{
        Some((TString, B(StringExpr(self.lexer.curr_string.clone()))))
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
            Token::Mul => {
                let (t, op2) = self.get_nxt_and_parse();
                //FIXME it's better to use a type-checker
                if t == TInt32{
                    return Some((TInt32, B(MulExpr(op1, op2))))
                }
                else{
                    panic!("Expected i32 as the type of rhs expression");
                }
            },
            Token::Div => {
                let (t, op2) = self.get_nxt_and_parse();
                //FIXME it's better to use a type-checker
                if t == TInt32{
                    return Some((TInt32, B(DivExpr(op1, op2))))
                }
                else{
                    panic!("Expected i32 as the type of rhs expression");
                }
            },
            Token::Equals => {
                let (t, op2) = self.get_nxt_and_parse();
                return Some((TVoid, B(EqualsExpr(op1, op2))))
            },
            Token::LessThan => {
                let (t, op2) = self.get_nxt_and_parse();
                return Some((TVoid, B(LessThanExpr(op1, op2))))
            },
            Token::GreaterThan => {
                let (t, op2) = self.get_nxt_and_parse();
                return Some((TVoid, B(GreaterThanExpr(op1, op2))))
            },
            Token::LessThanGreaterThan => {
                let (t, op2) = self.get_nxt_and_parse();
                return Some((TVoid, B(NotEqualsExpr(op1, op2))))
            },
            //FIXME ';', ')' can be a encountered as well. deal with it.
            _ => {
                return Some((TInt32, op1))
            }
        }
    }

    fn parse_function_decl(&mut self, decls : &mut Vec<Decl>){
        match self.lexer.get_token(){
            Token::Ident => {
                let id = self.lexer.curr_string.clone();

                //parse the parameters list
                let field_decs = self.parse_function_params_list();

                //parse return type
                let ret_type = self.parse_function_ret_type();

                //parse body here
                let e = self.expr();
                debug_assert!(e.is_some() == true, "Function body cannot be empty");
                let body = e.unwrap();

                //function id ( fieldDec; ) : tyId = exp
                decls.push(FunDec(id, field_decs, ret_type, body.1, body.0));
            },
            _ => panic!("Expected an id after 'function'")
        }
    }

    fn parse_function_params_list(&mut self) -> OptionalParamInfoList {
        match self.lexer.get_token(){
            Token::LeftParen => {
                let mut field_decs : Vec<(String, TType)> = Vec::new();
                loop{
                    match self.lexer.get_token() {
                        Token::Comma => continue,
                        Token::RightParen => { //parameterless function
                            break;
                        },
                        Token::Eof => panic!("Unexpected eof encountered. Expected a ')' after field-declaration."),
                        Token::Ident => {
                            let id = self.lexer.curr_string.clone();
                            //FIXME should we verify duplicate params here?
                            //HashMap and BTreeMap do not respect the order of insertions
                            //which is required to set up args during call.
                            //Vec will respect the order but cost O(n) for the verification
                            //Need multi_index kind of a structure from C++ Boost
                            if field_decs.iter().find(|&tup| tup.0 == id).is_some(){
                                panic!(format!("parameter '{}' found more than once", id));
                            }
                            match  self.lexer.get_token() {
                                Token::Colon => {
                                    match self.lexer.get_token() {
                                        Token::Int |
                                        Token::TokString |
                                        Token::Ident => {
                                            let ty = Self::get_ty_from_string(self.lexer.curr_string.as_str());
                                            field_decs.push((id, ty));
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

    fn parse_call_args(&mut self) -> OptionalTypeExprTupleList{
        let mut args_list  = Vec::new();
        while true {
            println!("loop");
            match self.lexer.get_token() {
                Token::RightParen => break,
                Token::Number |
                Token::Ident |
                Token::TokString => {
                    println!("{:?}", self.lexer.curr_token);
                    let e = self.expr();
                    if e.is_some() {
                        args_list.push(e.unwrap());
                    }
                },
                _ => {
                    panic!("Invalid expression used as a call argument");
                }
                //_ => panic!("Invalid expression used as a call argument")
            }

            if self.lexer.curr_token == Token::RightParen {
                break
            }
        }
        self.lexer.get_token();
        return if args_list.is_empty() {None} else {Some(args_list)}
    }

    fn parse_function_ret_type(&mut self) -> TType{
        match self.lexer.get_token() {
            Token::Colon => {
                match self.lexer.get_token() {
                    Token::Int |
                    Token::TokString |
                    Token::Ident => {
                        let ty = Self::get_ty_from_string(self.lexer.curr_string.as_str()); 
                        match self.lexer.get_token(){
                            Token::Equals => {self.lexer.get_token();},
                            _ => panic!("Expected '=' after the return type")
                        }
                        return ty                     
                    },
                    _ => panic!("Expected a type after ':'")
                }
            }
            Token::Equals => {
                self.lexer.get_token(); //eat '='
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

    fn parse_while_expr(&mut self) -> Option<(TType, B<Expr>)>{
        self.lexer.get_token();
        let opt_tup = self.expr().unwrap();
        //Because ident-expr parsing advances to the next token
        //and returns a TVoid, there is an extra check on the
        //curr_token
        if opt_tup.0 != TInt32 && self.lexer.curr_token != Token::Do{
            self.lexer.get_token();
        }
        match self.lexer.curr_token {
            Token::Do => {
                self.lexer.get_token();
                let (ty, body) = self.expr().unwrap();
                Some((ty, B(WhileExpr(opt_tup.1, body))))
            },
            _ => panic!("Expected 'do' after the while expression")
        }
    }

    fn parse_if_then_else_expr(&mut self) -> Option<(TType, B<Expr>)>{
        //eat 'if'
        self.lexer.get_token();
        //parse the conditional expr
        let opt_tup = self.expr().unwrap();
        //since only arithmetic expr parsing advances to point to the next token,
        //we do a typecheck in order to determine if we match on the curr_token
        //or call get_token()
        if opt_tup.0 != TInt32 && self.lexer.curr_token != Token::Then{
            self.lexer.get_token();
        }
        match self.lexer.curr_token {
            Token::Then => {
                self.lexer.get_token(); //advance to the next token
                let (_, then_expr) = self.expr().unwrap();
                match self.lexer.curr_token {
                    Token::Else => {
                        self.lexer.get_token(); //advance to the next token
                        let (_, else_body) = self.expr().unwrap();
                        return Some((TVoid, B(IfThenElseExpr(opt_tup.1, then_expr, else_body))))
                    }
                    t => {} //FIXME this isn't an if-then-else expr. should we do something here?
                }
                Some((TVoid, B(IfThenExpr(opt_tup.1, then_expr))))
            },
            _ => panic!("Expected then after the if expression")
        }
    }

    fn parse_for_expr(&mut self) -> Option<(TType, B<Expr>)>{
        match self.lexer.get_token(){
            Token::Ident => {
                let id = self.lexer.curr_string.clone();
                match self.lexer.get_token(){
                   Token::ColonEquals => {
                       self.lexer.get_token();
                       let (_, id_expr) = self.expr().unwrap();
                       match self.lexer.curr_token{
                           Token::To => {
                               self.lexer.get_token();
                               let (_, to_expr) = self.expr().unwrap();
                               match self.lexer.curr_token{
                                   Token::Do => {
                                       self.lexer.get_token();
                                       let (_, do_expr) = self.expr().unwrap();
                                       return Some((TVoid, B(ForExpr(id, id_expr, to_expr, do_expr))))
                                   },
                                   _ => panic!("Expected 'do' after expression")
                               }
                           },
                           _ => panic!("Expected 'to' after expression")
                       }

                   },
                   _ => panic!("Expected := after ident in a for construct")
                }
            },
            _ => panic!("Expected an ident after 'for'")
        }
    }
        
}

#[test]
fn test_func_decl_no_params() {
    let mut p = Parser::new("function foo()=print(\"ab\")".to_string());
    p.start_lexer();
    let mut decls = Vec::new();
    p.parse_function_decl(&mut decls);
    assert_eq!(decls.len(), 1);
    match &decls[0]{
        &FunDec(ref name, _, ref ty, ref b_expr, ref b_type) => {
            assert_eq!(String::from("foo"), *name);
            assert_eq!(TVoid, *ty);
            match &**b_expr {
                &CallExpr(ref name, _) => assert_eq!(String::from("print"), *name),
                _ => {}
            }
        },
        _ => {}
    }
}

#[test]
#[should_panic(expected="parameter 'a' found more than once")]
fn test_parse_function_params_list_duplicate_params() {
    let mut p = Parser::new("foo(a:int, a:int)".to_string());
    p.start_lexer();
    p.parse_function_params_list();
}

#[test]
fn test_parse_call_expr_num_expr(){
    let mut p = Parser::new("f(1)".to_string());
    p.start_lexer();
    let tup = p.expr();
    assert_eq!(tup.is_some(), true);
    let (ty, b_expr) = tup.unwrap();
    assert_eq!(ty, TVoid);
    match *b_expr {
        CallExpr(ref n, ref type_expr_lst) => {
            assert_eq!(n, "f");
            assert_eq!(type_expr_lst.is_some(), true);
            match type_expr_lst{
                &Some(ref l) => {
                    assert_eq!(l.len(), 1);
                    let (ref ty, ref b_expr) = l[0usize];
                    assert_eq!(*ty, TInt32);
                    match &**b_expr {
                        &NumExpr(n) => assert_eq!(n, 1),
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
fn test_parse_call_expr_ident_expr(){
    let mut p = Parser::new("f(abc)".to_string());
    p.start_lexer();
    let tup = p.expr();
    assert_eq!(tup.is_some(), true);
    let (ty, b_expr) = tup.unwrap();
    assert_eq!(ty, TVoid);
    match *b_expr {
        CallExpr(ref n, ref type_expr_lst) => {
            assert_eq!(n, "f");
            assert_eq!(type_expr_lst.is_some(), true);
            match type_expr_lst{
                &Some(ref l) => {
                    assert_eq!(l.len(), 1);
                    let (ref ty, ref b_expr) = l[0usize];
                    assert_eq!(*ty, TVoid);
                    match &**b_expr {
                        &IdExpr(ref id) => assert_eq!(*id, "abc"),
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
fn test_only_string_expr() {
    let mut p = Parser::new("\"abc\"".to_string());
    p.start_lexer();
    assert_eq!(p.lexer.curr_token, Token::TokString);
}

#[test]
fn test_parse_call_expr_string_arg(){
    let mut p = Parser::new("f(\"abc\")".to_string());
    p.start_lexer();
    let tup = p.expr();
    assert_eq!(tup.is_some(), true);
    let (ty, b_expr) = tup.unwrap();
    assert_eq!(ty, TVoid);
    match *b_expr {
        CallExpr(ref n, ref type_expr_lst) => {
            assert_eq!(n, "f");
            assert_eq!(type_expr_lst.is_some(), true);
            match type_expr_lst{
                &Some(ref l) => {
                    assert_eq!(l.len(), 1);
                    let (ref ty, ref b_expr) = l[0usize];
                    assert_eq!(*ty, TString);
                    match &**b_expr {
                        &StringExpr(ref value) => assert_eq!(*value, "abc"),
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
fn test_parse_call_expr_inum_ident_exprs(){
    let mut p = Parser::new("f(1, abc)".to_string());
    p.start_lexer();
    let tup = p.expr();
    assert_eq!(tup.is_some(), true);
    let (ty, b_expr) = tup.unwrap();
    assert_eq!(ty, TVoid);
    match *b_expr {
        CallExpr(ref n, ref type_expr_lst) => {
            assert_eq!(n, "f");
            assert_eq!(type_expr_lst.is_some(), true);
            match type_expr_lst{
                &Some(ref l) => {
                    assert_eq!(l.len(), 2);
                    let (ref ty, ref b_expr) = l[1usize];
                    assert_eq!(*ty, TVoid);
                    match &**b_expr {
                        &IdExpr(ref id) => assert_eq!(*id, "abc"),
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
fn test_parse_call_expr_add_expr(){
    let mut p = Parser::new("f(1+2)".to_string());
    p.start_lexer();
    let tup = p.expr();
    assert_eq!(tup.is_some(), true);
    let (ty, b_expr) = tup.unwrap();
    assert_eq!(ty, TVoid);
    match *b_expr {
        CallExpr(ref n, ref type_expr_lst) => {
            assert_eq!(n, "f");
            assert_eq!(type_expr_lst.is_some(), true);
            match type_expr_lst{
                &Some(ref l) => {
                    assert_eq!(l.len(), 1);
                    let (ref ty, ref b_expr) = l[0usize];
                    assert_eq!(*ty, TInt32);
                    match &**b_expr {
                        &AddExpr(ref op1, ref op2) => {
                            match &**op1 {
                                &NumExpr(n) => assert_eq!(n, 1),
                                _ => {}
                            }
                            match &**op2 {
                                &NumExpr(n) => assert_eq!(n, 2),
                                _ => {}
                            }
                        },
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
fn test_parse_call_expr_add_expr_with_call_expr_and_num_expr(){
    let mut p = Parser::new("f(a()+2)".to_string());
    p.start_lexer();
    let tup = p.expr();
    assert_eq!(tup.is_some(), true);
    let (ty, b_expr) = tup.unwrap();
    assert_eq!(ty, TVoid);
    match *b_expr {
        CallExpr(ref n, ref type_expr_lst) => {
            assert_eq!(n, "f");
            assert_eq!(type_expr_lst.is_some(), true);
            match type_expr_lst{
                &Some(ref l) => {
                    //assert_eq!(l.len(), 1);
                    let (ref ty, ref b_expr) = l[0usize];
                    //assert_eq!(*ty, TInt32);
                    match &**b_expr {
                        &AddExpr(ref op1, ref op2) => {
                            match &**op1 {
                                &NumExpr(n) => assert_eq!(n, 1),
                                _ => {}
                            }
                            match &**op2 {
                                &NumExpr(n) => assert_eq!(n, 2),
                                _ => {}
                            }
                        },
                        &NumExpr(_) => panic!("num expr") ,
                        &CallExpr(_, _) => panic!("callexpr"),
                        _ => {panic!("Different expr found")}
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }
}

#[test]
fn test_parse_call_expr_no_args(){
    let mut p = Parser::new("f()".to_string());
    p.start_lexer();
    let tup = p.expr();
    assert_eq!(tup.is_some(), true);
    let (ty, b_expr) = tup.unwrap();
    assert_eq!(ty, TVoid);
     match *b_expr {
        CallExpr(ref n, _) => assert_eq!(n, "f"),
        _ => {}
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
    let m = p.parse_function_params_list();
    assert_eq!(m, None);
}

#[test]
fn test_field_decs_one_dec(){
    let mut p = Parser::new("f(a: int)".to_string());
    p.start_lexer();
    let m = p.parse_function_params_list();
    assert_eq!(m.is_some(), true);
    assert_eq!(m.unwrap().len(), 1);
}

#[test]
fn test_field_decs_two_decs(){
    let mut p = Parser::new("f(a: int, b:int)".to_string());
    p.start_lexer();
    let m = p.parse_function_params_list();
    assert_eq!(m.is_some(), true);
    assert_eq!(m.unwrap().len(), 2);
}

#[test]
fn test_field_decs_two_decs_int_string(){
    let mut p = Parser::new("f(a: int, b:string)".to_string());
    p.start_lexer();
    let m = p.parse_function_params_list().unwrap();
    assert_eq!(m.len(), 2);
    assert_eq!(m[0].1, TType::TInt32);
    assert_eq!(m[1].1, TType::TString);
}

#[test]
fn test_field_decs_one_dec_with_alias(){
    let mut p = Parser::new("f(a: myint)".to_string());
    p.start_lexer();
    let m = p.parse_function_params_list().unwrap();
    assert_eq!(m[0].1, TType::TCustom("myint".to_string()));
}

#[test]
#[should_panic(expected="Unexpected eof encountered. Expected a ')' after field-declaration.")]
fn test_field_decs_no_closing_paren(){
    let mut p = Parser::new("f(a: myint".to_string());
    p.start_lexer();
    p.parse_function_params_list();
}

#[test]
fn test_let_var_decl_returns_block() {
    let mut p = Parser::new("let var a : int := 1 in 1+1 end".to_string());
    assert_eq!(p.run().is_some(), true);
}

#[test]
fn test_let_var_decl_returns_let_expr() {
    let mut p = Parser::new("let var a : int := 1 in a end".to_string());
    let b = p.run().unwrap();
    match *b.expr.unwrap(){
        LetExpr(ref v, ref o) => {
            assert_eq!(v.len(), 1);
            assert_eq!(o.is_some(), true);
            match v[0]{
                VarDec(ref id, ref ty, ref e) => {
                    assert_eq!(*id, "a".to_string());
                    match **e{ //**e means deref deref B<T> which results in T
                        NumExpr(n) => assert_eq!(1, n),
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
    let mut p = Parser::new("let var a : int := 1 in a end".to_string());
    let b = p.run().unwrap();
    assert_eq!(b.sym_tab.borrow().len(), 1);
    assert_eq!(b.sym_tab.borrow().get(&"a".to_string()), Some(&TInt32));
}

#[test]
fn test_let_add_expr() {
    let mut p = Parser::new("let var a : int := 1 + 3 + 1 in a end".to_string());
    let b = p.run().unwrap();
    match *b.expr.unwrap(){
        LetExpr(ref v, ref o) => {
            assert_eq!(v.len(), 1);
            assert_eq!(o.is_some(), true);
            match v[0]{
                VarDec(ref id, ref ty, ref e) => {
                    assert_eq!(*id, "a".to_string());
                    match **e{ //**e means deref deref B<T> which results in T
                        AddExpr(ref e1, ref e2) => {
                            match **e1{
                                NumExpr(n) => assert_eq!(n, 1),
                                _ => panic!("num expr expected")
                            }

                            match **e2{
                                AddExpr(ref e1, ref e2) => {
                                    match **e1{
                                        NumExpr(n) => assert_eq!(n, 3),
                                        _ => panic!("num expr expected")
                                    }

                                    match **e2{
                                        NumExpr(n) => assert_eq!(n, 1),
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
    let mut p = Parser::new("let var a : int := 1\nvar b : int:=2\n in b end".to_string());
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
                NumExpr(n) => {
                    assert_eq!(n, 1);
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
                        NumExpr(n) => assert_eq!(n, 5),
                        _ => {}
                    }
                    match **e2 {
                        NumExpr(n) => assert_eq!(n, 16),
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

#[test]
fn test_if_then_expr(){
    let mut p = Parser::new("if 1 then 1".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenExpr(ref conditional_expr, ref then_expr) => {
            match(**conditional_expr){
                NumExpr(n) => assert_eq!(n, 1),
                _ => {}
            }
        },
        _ => {}
    } 
}

#[test]
fn test_if_then_equality_as_conditional_expr(){
    let mut p = Parser::new("if 1=1 then 1".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenExpr(ref conditional_expr, ref then_expr) => {
            match(**conditional_expr){
                EqualsExpr(ref e1, ref e2) => {},
                _ => panic!("Expected an equals expr")
            }
        },
        _ => panic!("Expected an if-then expr")
    } 
}
#[test]
fn test_if_then_with_ident_as_conditional_expr(){
    let mut p = Parser::new("if a then 1".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenExpr(ref conditional_expr, _) => {
            match(**conditional_expr){
               IdExpr(ref i) => assert_eq!(*i, String::from("a")),
                _ => {}
            }
        },
        _ => {}
    } 
}

#[test]
fn test_if_then_with_ident_involving_equality_test_conditional_expr(){
    let mut p = Parser::new("if a=1 then 1".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenExpr(ref conditional_expr, _) => {
            match(**conditional_expr){
               EqualsExpr(ref e1, ref e2) => {},
                _ => panic!("Expected equality expression")
            }
        },
        _ => panic!("Expected if-then expression")
    } 
}
#[test]
fn test_if_then_with_add_as_conditional_expr(){
    let mut p = Parser::new("if 1+1 then 1".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenExpr(ref conditional_expr, ref then_expr) => {
            match(**conditional_expr){
                AddExpr(ref l, ref r) => {
                    match **l{
                        NumExpr(n) => assert_eq!(n, 1),
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
fn test_if_then_with_string_as_conditional_expr(){
    let mut p = Parser::new("if \"abhi\" then 1".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenExpr(ref conditional_expr, ref then_expr) => {
            match(**conditional_expr){
                StringExpr(ref s) => assert_eq!(*s, String::from("abhi")),
                _ => {}
            }
        },
        _ => {}
    } 
}
#[test]
fn test_if_then_else_expr(){
    let mut p = Parser::new("if 1 then foo() else foo()".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenElseExpr(ref conditional_expr, ref then_expr, ref else_expr) => {
            match(**conditional_expr){
                NumExpr(n) => assert_eq!(n, 1),
                _ => {}
            }
            match(**else_expr){
                CallExpr(ref fn_name, _) => assert_eq!(*fn_name, String::from("foo")),
                _ => panic!("not covered")
            }
        },
        _ => {panic!("bingo!")}
    } 

}

#[test]
fn test_if_then_else_expr_with_num_expressions(){
    let mut p = Parser::new("if 1 then 1 else 0".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenElseExpr(ref conditional_expr, ref then_expr, ref else_expr) => {
            match(**conditional_expr){
                NumExpr(n) => assert_eq!(n, 1),
                _ => {panic!("Unexpected expression")}
            }
            match(**else_expr){
                NumExpr(n) => assert_eq!(n, 0),
                _ => panic!("not covered")
            }
        },
        _ => {panic!("bingo!")}
    } 

}
#[test]
fn test_if_expr_with_string_expr_as_conditional_expr(){
    let mut p = Parser::new("if \"abhi\" then 1".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenExpr(ref conditional_expr, _) => match(**conditional_expr) {
                StringExpr(ref s) => assert_eq!(*s, "abhi"),
                _ =>  panic!("This will not exhecute")
        },
        _ => panic!("This will not execute")
    } 
}
#[test]
#[should_panic(expected="Type mismatch between the then and else expressions")]
fn test_if_then_else_expr_fail_string_return(){
    let mut p = Parser::new("if 1 then 1 else \"abhi\"".to_string());
    p.start_lexer();
    
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        IfThenElseExpr(_, _, ref else_expr) => {
            match(**else_expr){
                StringExpr(_) => panic!("Type mismatch between the then and else expressions"),
                _ =>  panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}

#[test]
fn test_while_expr(){
    let mut p = Parser::new("while 1 do 1".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        WhileExpr(ref conditional_expr, ref do_expr) => {
            match(**conditional_expr){
                NumExpr(n) => assert_eq!(n, 1),
                _ => panic!("This will not execute")
            }
            match(**do_expr){
                NumExpr(n) => assert_eq!(n, 1),
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}

#[test]
fn test_while_expr_with_string_as_conditional_expr(){
    let mut p = Parser::new("while \"abhi\" do 1".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        WhileExpr(ref conditional_expr, ref do_expr) => {
            match(**conditional_expr){
                StringExpr(ref s) => assert_eq!(*s, "abhi"),
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}

#[test]
fn test_while_expr_with_addexpr_as_conditional_expr(){
    let mut p = Parser::new("while 1+1 do 1".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        WhileExpr(ref conditional_expr, ref do_expr) => {
            match(**conditional_expr){
                AddExpr(ref l, ref r) => {
                    match **l{
                        NumExpr(n) => assert_eq!(n, 1),
                        _ => {}
                    }
                },
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}

#[test]
fn test_while_expr_with_less_than_cmp_as_conditional_expr(){
    let mut p = Parser::new("while 1<1 do 1".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        WhileExpr(ref conditional_expr, ref do_expr) => {
            match(**conditional_expr){
                LessThanExpr(ref l, ref r) => {
                    match **l{
                        NumExpr(n) => assert_eq!(n, 1),
                        _ => {}
                    }
                },
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}

#[test]
fn test_while_expr_with_greater_than_cmp_as_conditional_expr(){
    let mut p = Parser::new("while 1>1 do 1".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        WhileExpr(ref conditional_expr, ref do_expr) => {
            match(**conditional_expr){
                GreaterThanExpr(ref l, ref r) => {
                    match **l{
                        NumExpr(n) => assert_eq!(n, 1),
                        _ => {}
                    }
                },
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}
#[test]
fn test_while_expr_with_ident_as_conditional_expr(){
    let mut p = Parser::new("while a do 1".to_string());
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        WhileExpr(ref conditional_expr, ref do_expr) => {
            match(**conditional_expr){
                IdExpr(ref id) => {
                    assert_eq!(*id, String::from("a"));
                },
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}
#[test]
fn test_for_expr(){
    let mut p = Parser::new("for id:= 1 to 10 do 1+1".to_string()); 
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        ForExpr(ref id, ref from_expr, ref to_expr, ref do_expr) => {
            assert_eq!(*id, String::from("id"));
            match(**from_expr){
                NumExpr(n) => assert_eq!(n, 1),
                _ => panic!("This will not execute")
            }
            match(**to_expr){
                NumExpr(n) => assert_eq!(n, 10),
                _ => panic!("This will not execute")
            }
            match(**do_expr){
                AddExpr(ref l, ref r) => {
                    match(**l){
                        NumExpr(n) => assert_eq!(n, 1),
                        _ => {}
                    }
                },
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}

#[test]
fn test_for_expr_with_ident_as_from_expr(){
    let mut p = Parser::new("for id:= a to 10 do 1+1".to_string()); 
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        ForExpr(ref id, ref from_expr, _, _) => {
            match(**from_expr){
                IdExpr(ref i) => assert_eq!(*i, String::from("a")),
                _ => panic!("this will not execute")
            }
        },
        _ => panic!("this will not execute")
    } 
}
#[test]
fn test_for_expr_with_ident_as_to_and_from_expr(){
    let mut p = Parser::new("for id:= a to b do 1+1".to_string()); 
    p.start_lexer();
    let (ty, expr) = p.expr().unwrap();
    match(*expr){
        ForExpr(ref id, ref from_expr, ref to_expr, _) => {
            match(**to_expr){
                IdExpr(ref i) => assert_eq!(*i, String::from("b")),
                _ => panic!("This will not execute")
            }
            match(**from_expr){
                IdExpr(ref i) => assert_eq!(*i, String::from("a")),
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
    match(*expr){
        ForExpr(ref id, ref from_expr, ref to_expr, _) => {
            match(**to_expr){
                IdExpr(ref i) => assert_eq!(*i, String::from("b")),
                _ => panic!("This will not execute")
            }
            match(**from_expr){
                IdExpr(ref i) => assert_eq!(*i, String::from("a")),
                _ => panic!("This will not execute")
            }
        },
        _ => panic!("This will not execute")
    } 
}

