#[derive(Copy, Clone, PartialEq)]
//Copy and Clone - this is to enable returning of clone of enum members and hence avoid getting `moving-borrow` errors
//PartialEq - this is to enable comparison during unit tests, etc.
#[derive(Debug)]
pub enum Token{
    //keywords
    Array,
    Rec,
    Break,
    Do,
    End,
    While,
    If,
    Then,
    Else,
    For,
    In,
    Let,
    Of,
    To,
    Type,
    Var,
    Function,
    Nil,
    Int,
    TokString,

    //structural symbols
    Plus,
    Minus,
    Mul,
    Div,
    LogAnd,
    LogNot,
    LogOr,
    Equals,                // =
    LessEquals,            // <=
    GreaterEquals,         // >=
    LessThan,
    GreaterThan,
    LessThanGreaterThan,
    ColonEquals,           // :=
    LeftParen,
    RightParen,
    LeftCurly,
    RightCurly,
    LeftSquare,
    RightSquare,
    SemiColon,
    Colon,
    Comma,
    Dot,

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
