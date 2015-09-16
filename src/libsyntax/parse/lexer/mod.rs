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
        let mut l = Lexer{ src_code : src_code.as_bytes().to_vec(), line_pos : 1, ..Default::default()};
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

    pub fn get_token(&mut self) -> Token{
        //do not loop over the match
        //this will cause a problem for ident storing (curr_string.clear())
        match self.curr_char{
            '+' => { self.curr_token = Token::Plus; self.get_char(); return self.curr_token},
            '-' => { self.curr_token = Token::Minus; self.get_char(); return self.curr_token},
            '*' => { self.curr_token = Token::Mul; self.get_char(); return self.curr_token},
            '&' => { self.curr_token = Token::LogAnd; self.get_char(); return self.curr_token},
            '|' => { self.curr_token = Token::LogOr; self.get_char(); return self.curr_token},
            '>' => {
                self.curr_token =
                if self.curr_char == '=' {
                    Token::GreaterEquals
                }
                else{
                    Token::GreaterThan
                };
                self.get_char();
                return self.curr_token
             },
            '<' => {
                self.curr_token =
                if self.curr_char == '>' {
                    Token::LessThanGreaterThan
                }
                else if self.curr_char == '=' {
                    Token::LessEquals
                }
                else{
                    Token::LessThan
                };
                self.get_char();
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
            '{' => { self.curr_token = Token::LeftCurly; self.get_char(); return self.curr_token},
            '}' => { self.curr_token = Token::RightCurly; self.get_char(); return self.curr_token},
            '[' => { self.curr_token = Token::LeftSquare; self.get_char(); return self.curr_token},
            ']' => { self.curr_token = Token::RightSquare; self.get_char(); return self.curr_token},
            '(' => { self.curr_token = Token::LeftParen; self.get_char(); return self.curr_token},
            ')' => { self.curr_token = Token::RightParen; self.get_char(); return self.curr_token},
            ',' => { self.curr_token = Token::Comma; self.get_char(); return self.curr_token},
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
                let token = self.match_token();

            }
        }
    }

    fn match_token(&self)->Token{
        match &*self.curr_string{
           "let"      => Token::Let,
           "var"      => Token::Var,
           "array"    => Token::Array,
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
}
