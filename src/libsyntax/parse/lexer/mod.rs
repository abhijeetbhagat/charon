#![allow(dead_code)]

use parse::tokens::*;

#[derive(Default)]
pub struct Lexer{
    pub curr_token : Token,
    next_token : Token,
    curr_char : char,
    pub curr_string : String,
    src_code : Vec<u8>,
    char_pos : usize,
    pub line_pos : usize
}

impl Lexer{
    pub fn new(src_code : String)->Self{
        let l = Lexer{ src_code : src_code.as_bytes().to_vec(), line_pos : 1, ..Default::default()};
        //l.get_char();
        l
    }

    //FIXME: get_char() shouldn't be exposed
    pub fn get_char(&mut self){
        if self.char_pos < self.src_code.len() {
            self.curr_char = self.src_code[self.char_pos] as char;
            self.char_pos += 1;
        }
        else{
            self.curr_char = '\0';
        }
    }

    pub fn peek_next(&mut self) -> Token{
        //save context
        let old_pos = self.char_pos.clone();
        let old_tok = self.curr_token.clone();
        let old_string = self.curr_string.clone(); 
        let t = self.get_token();
        //load saved context
        self.char_pos = old_pos;
        self.curr_token = old_tok;
        self.curr_string = old_string;
        t
    }

    pub fn get_token(&mut self) -> Token{
        //do not loop over the match
        //this will cause a problem for ident storing (curr_string.clear())
        macro_rules! get_cur_tok_and_eat{
            //without the double curly braces, compiler complains saying everything after the
            //first statement will be ignored. So we tell it to treat the body as a block
            ($e : path) => {{self.curr_token = $e; self.get_char(); return self.curr_token;}}
        }
        match self.curr_char{
            '+' => { get_cur_tok_and_eat!(Token::Plus)},
            '-' => { get_cur_tok_and_eat!(Token::Minus)},
            '*' => { get_cur_tok_and_eat!(Token::Mul)},
            '/' => { get_cur_tok_and_eat!(Token::Div)},
            '&' => { get_cur_tok_and_eat!(Token::LogAnd)},
            '|' => { get_cur_tok_and_eat!(Token::LogOr)},
            '>' => {
                self.get_char();
                self.curr_token = Token::GreaterThan;
                if self.curr_char == '=' {
                    self.get_char();
                    self.curr_token = Token::GreaterEquals;
                }
                return self.curr_token
             },
            '<' => {
                self.get_char();
                self.curr_token =
                if self.curr_char == '>' {
                    self.get_char();
                    Token::LessThanGreaterThan
                }
                else if self.curr_char == '=' {
                    self.get_char();
                    Token::LessEquals
                }
                else{
                    Token::LessThan
                };
                return self.curr_token
            },
            '.' =>{
                self.curr_token = Token::Dot;
                self.get_char();
                return self.curr_token
            },
            '=' => {
                self.get_char();
                self.curr_token = Token::Equals;
                return self.curr_token
            },
            '{' => { get_cur_tok_and_eat!(Token::LeftCurly)}, 
            '}' => { get_cur_tok_and_eat!(Token::RightCurly)},
            '[' => { get_cur_tok_and_eat!(Token::LeftSquare)},
            ']' => { get_cur_tok_and_eat!(Token::RightSquare)},
            '(' => { get_cur_tok_and_eat!(Token::LeftParen)},
            ')' => { get_cur_tok_and_eat!(Token::RightParen)},
            ',' => { get_cur_tok_and_eat!(Token::Comma)},
            ':' => {
                self.get_char();
                self.curr_token =
                 if self.curr_char == '=' {
                     self.get_char();
                    Token::ColonEquals
                 }
                 else {
                    Token::Colon
                 };
                 return self.curr_token
             },
            ';' => { self.curr_token = Token::SemiColon; self.get_char(); return self.curr_token},
            '"' => {
                self.curr_string.clear();
                loop {
                    self.get_char();
                    if self.curr_char == '\\' {
                        self.get_char();
                        match self.curr_char {
                            'n' => self.curr_string.push('\n'),
                            't' => self.curr_string.push('\t'),
                            '"' => self.curr_string.push('"'),
                            '\\' => self.curr_string.push('\\'),
                            _ => panic!("Unrecognized escape sequence")
                        }
                        //self.curr_string.push(self.curr_char);
                        continue;
                    }

                    if self.curr_char == '"' {
                        self.get_char(); //eat "
                        break;
                    }

                    if self.curr_char == '\0' {
                        panic!("Unexpected eof. Expected a closing '\"'.");
                    }

                    self.curr_string.push(self.curr_char);
                }

                self.curr_token = Token::TokString;
                return self.curr_token;
            },
            '0' ... '9' => {
                self.curr_string.clear();
                self.curr_string.push(self.curr_char);
                self.get_char();
                while self.curr_char.is_digit(10) && self.curr_char != '\0' {
                    self.curr_string.push(self.curr_char);
                    self.get_char();
                }
                self.curr_token = Token::Number;
                return self.curr_token
            },
            'a' ... 'z' => {
                self.curr_string.clear();
                self.curr_string.push(self.curr_char);
                self.get_char();
                while self.curr_char.is_alphanumeric() && self.curr_char != '\0' {
                    self.curr_string.push(self.curr_char);
                    self.get_char();
                }

                self.curr_token = self.match_token(); //mat(&self.curr_string) {return Token::} else {return Token::Ident}
                return self.curr_token

            },
            //\n is also whitespace. So put it before whitespace check
            '\0' => {self.curr_token = Token::Eof; return self.curr_token},
            '\n' => { self.line_pos += 1; self.curr_token = Token::NewLine; self.get_char(); return self.curr_token },
            c if c.is_whitespace() => {
                loop{
                    self.get_char();
                    if !self.curr_char.is_whitespace(){
                        break;
                    }
                }
                self.curr_token = self.get_token();
                return self.curr_token
            },

            _ => {self.curr_token = Token::Error; return self.curr_token}
        }
    }

    fn run(&mut self){
        while self.char_pos < self.src_code.len(){
            self.get_char();
            if !self.curr_char.is_whitespace(){
                self.get_char();
            }
            else{
                let _ = self.match_token();

            }
        }
    }

    fn match_token(&self)->Token{
        match &*self.curr_string{
           "let"      => Token::Let,
           "var"      => Token::Var,
           "array"    => Token::Array,
           "rec"    => Token::Rec,
           "of"       => Token::Of,
           "type"      => Token::Type,
           "break"      => Token::Break,
           "do"      => Token::Do,
           "end"     => Token::End,
           "while"     => Token::While,
           "if"     => Token::If,
           "then"     => Token::Then,
           "else"     => Token::Else,
           "for"     => Token::For,
           "in"     => Token::In,
           "function"     => Token::Function,
           "nil"     => Token::Nil,
           "int"    => Token::Int,
           "string" => Token::TokString,
           "to" => Token::To,
            _ => Token::Ident
        }
    }
}

#[cfg(test)]
mod tests {
    use parse::tokens::*;
    use super::*; //use stuff thats in the file but outside this module

    #[test]
    fn test_match_token_binary_exp_nums(){
        let mut l = Lexer::new("1234+23451".to_string());
        l.get_char();
        assert!(l.get_token() == Token::Number);
        assert!(l.get_token() == Token::Plus);
        assert!(l.get_token() == Token::Number);
    }

    #[test]
    fn test_match_token_binary_exp_vars(){
        let mut l = Lexer::new("a+a".to_string());
        l.get_char();
        assert!(l.get_token() == Token::Ident);
        assert!(l.get_token() == Token::Plus);
        assert!(l.get_token() == Token::Ident);
    }

    #[test]
    fn test_match_token_binary_exp_mixed(){
        let mut l = Lexer::new("a+1".to_string());
        l.get_char();
        assert!(l.get_token() == Token::Ident);
        assert!(l.get_token() == Token::Plus);
        assert!(l.get_token() == Token::Number);
    }

    #[test]
    fn test_match_token_binary_exp_mixed_multiple_terms(){
        let mut l = Lexer::new("a+1+a".to_string());
        l.get_char();
        assert!(l.get_token() == Token::Ident);
        assert!(l.get_token() == Token::Plus);
        assert!(l.get_token() == Token::Number);
        assert!(l.get_token() == Token::Plus);
        assert!(l.get_token() == Token::Ident);
    }

    #[test]
    fn test_get_char(){
        let mut l = Lexer::new("1+1".to_string());
        l.get_char();
        assert!(l.curr_char == '1');
        l.get_char();
        assert!(l.curr_char == '+');
    }

    #[test]
    fn test_match_newline(){
        let mut l = Lexer::new("\n".to_string());
        l.get_char();
        assert!(l.get_token() == Token::NewLine);
        assert!(l.get_token() == Token::Eof);
    }

    #[test]
    fn test_line_pos(){
        let mut l = Lexer::new("\n\n\n".to_string());
        l.get_char();
        l.get_token();
        assert!(l.line_pos == 2);
        l.get_token();
        assert!(l.line_pos == 3);
        l.get_token();
        assert!(l.line_pos == 4);
    }

    #[test]
    fn test_let_block(){
        let mut l = Lexer::new("let var a : int := 1 in end".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Let);
        assert_eq!(l.get_token(), Token::Var);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Colon);
        assert_eq!(l.get_token(), Token::Int);
        assert_eq!(l.get_token(), Token::ColonEquals);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::In);
        assert_eq!(l.get_token(), Token::End);
    }

    #[test]
    fn test_let_block_with_2_var_decls(){
        let mut l = Lexer::new("let var a : int := 1\nvar b : int:=2\n in end".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Let);
        assert_eq!(l.get_token(), Token::Var);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Colon);
        assert_eq!(l.get_token(), Token::Int);
        assert_eq!(l.get_token(), Token::ColonEquals);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::NewLine);
        assert_eq!(l.get_token(), Token::Var);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Colon);
        assert_eq!(l.get_token(), Token::Int);
        assert_eq!(l.get_token(), Token::ColonEquals);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::NewLine);
        assert_eq!(l.get_token(), Token::In);
        assert_eq!(l.get_token(), Token::End);
    }

    #[test]
    fn test_let_without_spaces() {
        let mut l = Lexer::new("let var a:int := 1 in end".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Let);
        assert_eq!(l.get_token(), Token::Var);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Colon);
        assert_eq!(l.get_token(), Token::Int);
        assert_eq!(l.get_token(), Token::ColonEquals);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::In);
        assert_eq!(l.get_token(), Token::End);
    }

    #[test]
    fn test_type_dec_int() {
        let mut l = Lexer::new("type a = int".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Type);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Equals);
        assert_eq!(l.get_token(), Token::Int);
    }

    #[test]
    fn test_type_dec_array_of_int() {
        let mut l = Lexer::new("type ai = array of int".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Type);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Equals);
        assert_eq!(l.get_token(), Token::Array);
        assert_eq!(l.get_token(), Token::Of);
        assert_eq!(l.get_token(), Token::Int);
    }

    #[test]
    fn test_field_dec_int() {
        let mut l = Lexer::new("(a:int)".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::LeftParen);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Colon);
        assert_eq!(l.get_token(), Token::Int);
    }

    #[test]
    fn test_call_expr() {
        let mut l = Lexer::new("f(1)".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::LeftParen);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::RightParen);
    }

    #[test]
    fn test_call_expr_string_arg() {
        let mut l = Lexer::new("f(\"abc\")".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::LeftParen);
        assert_eq!(l.get_token(), Token::TokString);
        assert_eq!(l.get_token(), Token::RightParen);
    }


    #[test]
    fn test_simple_string() {
        let mut l = Lexer::new("\"abhi\"".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::TokString);
        assert_eq!(l.curr_string, "abhi".to_string());
    }

    #[test]
    fn test_string_with_escaped_double_quote() {
        let mut l = Lexer::new("\"ab\\\"hi\"".to_string());
        l.get_char();
        l.get_token();
        assert_eq!(l.curr_string, "ab\"hi".to_string());
    }

    #[test]
    fn test_string_with_escaped_backslash() {
        let mut l = Lexer::new("\"ab\\\\\"".to_string());
        l.get_char();
        l.get_token();
        assert_eq!(l.curr_string, "ab\\".to_string());
    }

    #[test]
    fn test_if_then(){
        let mut l = Lexer::new("if 1 then 1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::If);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Then);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test]
    fn test_if_then_else(){
        let mut l = Lexer::new("if 1 then 1 else 0".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::If);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Then);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Else);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test]
    fn test_while_block(){
        let mut l = Lexer::new("while 1 do 1+1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::While);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Do);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Plus);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test] 
    fn test_for_loop(){
        let mut l = Lexer::new("for id := 1 to 9 do 1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::For);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::ColonEquals);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::To);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Do);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test] 
    fn test_less_than_expr(){
        let mut l = Lexer::new("1 <1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::LessThan);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test] 
    fn test_less_than_equals_expr(){
        let mut l = Lexer::new("1 <=1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::LessEquals);
        assert_eq!(l.get_token(), Token::Number);
    }
    
    #[test] 
    fn test_greater_than_equals_expr(){
        let mut l = Lexer::new("1 >=1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::GreaterEquals);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test] 
    fn test_greater_than_expr(){
        let mut l = Lexer::new("1 >1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::GreaterThan);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test] 
    fn test_not_equals_expr(){
        let mut l = Lexer::new("1 <> 1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::LessThanGreaterThan);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test] 
    fn test_equals_expr(){
        let mut l = Lexer::new("1 = 1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Equals);
        assert_eq!(l.get_token(), Token::Number);
    }
    #[test] 
    fn test_peek(){
        let mut l = Lexer::new("1 <> 1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.peek_next(), Token::LessThanGreaterThan);
        assert_eq!(l.peek_next(), Token::LessThanGreaterThan);
        assert_eq!(l.get_token(), Token::LessThanGreaterThan);
        assert_eq!(l.get_token(), Token::Number);
    }
    #[test] 
    fn test_div_expr(){
        let mut l = Lexer::new("1 / 1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Div);
        assert_eq!(l.get_token(), Token::Number);
    }
    #[test] 
    fn test_div_expr_without_spaces(){
        let mut l = Lexer::new("1/1".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::Div);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test] 
    fn test_function_call_with_two_args(){
        let mut l = Lexer::new("f(a()+1)".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::LeftParen);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::LeftParen);
        assert_eq!(l.get_token(), Token::RightParen);
        assert_eq!(l.get_token(), Token::Plus);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test] 
    fn test_function_call_with_two_args_2(){
        let mut l = Lexer::new("f(a()+1)".to_string());
        l.get_char();
        loop{
            match l.get_token(){
                Token::Ident => println!("{:?}", l.curr_token),
                Token::LeftParen => println!("{:?}", l.curr_token),
                Token::RightParen => println!("{:?}", l.curr_token),
                Token::Plus => println!("{:?}", l.curr_token),
                Token::Number => println!("{:?}", l.curr_token),
                _ => break
            }
        }
    }

    #[test]
    fn test_array_decl(){
        let mut l = Lexer::new("array of int[3] of 0".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Array);
        assert_eq!(l.get_token(), Token::Of);
        assert_eq!(l.get_token(), Token::Int);
        assert_eq!(l.get_token(), Token::LeftSquare);
        assert_eq!(l.get_token(), Token::Number);
        assert_eq!(l.get_token(), Token::RightSquare);
        assert_eq!(l.get_token(), Token::Of);
        assert_eq!(l.get_token(), Token::Number);
    }

    #[test]
    fn test_typdef_decl(){
        let mut l = Lexer::new("type rec = {a:int}".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Type);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Equals);
        assert_eq!(l.get_token(), Token::LeftCurly);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Colon);
        assert_eq!(l.get_token(), Token::Int);
        assert_eq!(l.get_token(), Token::RightCurly);
    }

    #[test]
    fn test_record_decl(){
        let mut l = Lexer::new("rec{a:int}".to_string());
        l.get_char();
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::LeftCurly);
        assert_eq!(l.get_token(), Token::Ident);
        assert_eq!(l.get_token(), Token::Colon);
        assert_eq!(l.get_token(), Token::Int);
        assert_eq!(l.get_token(), Token::RightCurly);
    }
}
