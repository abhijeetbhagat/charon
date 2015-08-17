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
            '/' => { self.curr_token = if self.curr_char != '/' {Token::Div} else {Token::SlashSlash}; return self.curr_token},
            '^' => { self.curr_token = Token::Caret; self.get_char(); return self.curr_token},
            '%' => { self.curr_token = Token::Mod; self.get_char(); return self.curr_token},
            '&' => { self.curr_token = Token::LogAnd; self.get_char(); return self.curr_token},
            '~' => { self.curr_token = if self.curr_char != '=' {Token::NotEquals} else {Token::LogNot}; return self.curr_token},
            '|' => { self.curr_token = Token::LogOr; self.get_char(); return self.curr_token},
            '>' => {
                self.curr_token =
                if self.curr_char == '>' {
                    Token::RightShift
                } 
                else if self.curr_char == '=' {
                    Token::GreaterAssign
                }
                else{
                    Token::GreaterThan
                };
                self.get_char(); 
                return self.curr_token                    
             },
            '<' => {
                self.curr_token = 
                if self.curr_char == '<' {
                    Token::LeftShift
                } 
                else if self.curr_char == '=' {
                    Token::LessAssign
                }
                else{
                    Token::LessThan
                };
                self.get_char(); 
                return self.curr_token
            },
            '.' =>{
                self.curr_token = 
                if self.curr_char == '.'{
                    self.get_char();
                    if self.curr_char == '.'{Token::DotDotDot} else{Token::DotDot}
                }
                else{
                    Token::Dot
                };
                self.get_char();
                return self.curr_token                    
            },
            '=' => {
                self.get_char();
                self.curr_token = if self.curr_char == '=' {Token::Equals} else {Token::Assign};
                return self.curr_token                                        
            },
            '#' => { self.curr_token = Token::Hash; self.get_char(); return self.curr_token},
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
                 if self.curr_char == ':' {
                    Token::ColonColon
                 } 
                 else {
                    Token::Colon
                 };
                 self.get_char();
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
           "break"      => Token::Break,
           "goto"      => Token::Goto,
           "do"      => Token::Do,
           "end"     => Token::End,
           "while"     => Token::While,
           "repeat"  =>  Token::Repeat,
           "until"     => Token::Until,
           "if"     => Token::If,
           "then"     => Token::Then,
           "elseif"     => Token::ElseIf,
           "else"     => Token::Else,
           "for"     => Token::For,
           "in"     => Token::In,
           "function"     => Token::Function,
           "local"     => Token::Local,
           "return"     => Token::Return,
           "or"     => Token::Or,
           "nil"     => Token::Nil,
           "true"     => Token::True,
           "false"     => Token::False,
           "and"     => Token::And,       
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
}