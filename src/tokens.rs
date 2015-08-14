#[derive(Copy, Clone, PartialEq)] 
//Copy and Clone - this is to enable returning of clone of enum members and hence avoid getting `moving-borrow` errors
//PartialEq - this is to enable comparison during unit tests, etc.
pub enum Token{
    //keywords
    Break,
    Goto,
    Do,
    End,
    While,
    Repeat,
    Until,
    If,
    Then,
    ElseIf,
    Else,
    For,
    In,
    Function,
    Local,
    Return,
    Or,
    Nil,
    True,
    False,
    And,
    
    //structural symbols
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Caret,
    Hash,
    LogAnd,
    LogNot,
    LogOr,
    LeftShift,
    RightShift,
    SlashSlash,
    Equals,
    NotEquals,
    LessAssign,
    GreaterAssign,
    LessThan,
    GreaterThan,
    Assign,
    LeftParen,
    RightParen,
    LeftCurly,
    RightCurly,
    LeftSquare,
    RightSquare,
    ColonColon,
    SemiColon,
    Colon,
    Comma,
    Dot,
    DotDot,
    DotDotDot,
    
    //error/init.eof
    Error,
    NoToken,
    Eof,
    NewLine,
    
    Ident,
    Number
}

impl Default for Token{
    fn default()->Token{
        Token::NoToken
    }
}
