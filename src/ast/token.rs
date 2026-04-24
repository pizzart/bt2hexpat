use std::{fmt, str::FromStr};

use crate::ast::{data_type::DataType, literal::Literal};
use crate::str_enum;

str_enum! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Keyword {
        Auto => "auto",
        Break => "break",
        Case => "case",
        Const => "const",
        Continue => "continue",
        Default => "default",
        Do => "do",
        Else => "else",
        Enum => "enum",
        Extern => "extern",
        For => "for",
        Goto => "goto",
        If => "if",
        Local => "local",
        Register => "register",
        Return => "return",
        Signed => "signed",
        Sizeof => "sizeof",
        Static => "static",
        Struct => "struct",
        Switch => "switch",
        Typedef => "typedef",
        Union => "union",
        Unsigned => "unsigned",
        Volatile => "volatile",
        While => "while",
        {
            DataType(datatype: DataType) => {
                datatype.to_string(),
                (s if let Ok(s) = s.parse::<DataType>()) => Ok(Self::DataType(s)),
            },
        }
    }
}

str_enum! {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Punctuator {
        Ampersand => "&",
        And => "&&",
        Arrow => "->",
        Assign => "=",
        Asterisk => "*",
        BitAndAssign => "&=",
        BitLeftShift => "<<",
        BitLeftShiftAssign => "<<=",
        BitNot => "~",
        BitOr => "|",
        BitOrAssign => "|=",
        BitRightShift => ">>",
        BitRightShiftAssign => ">>=",
        BitXor => "^",
        BitXorAssign => "^=",
        Colon => ":",
        Comma => ",",
        Dec => "--",
        Div => "/",
        DivAssign => "/=",
        Dot => ".",
        Equal => "==",
        GreaterEqual => ">=",
        Hash => "#",
        Inc => "++",
        LAngledBracket => "<",
        LBrace => "{",
        LBracket => "[",
        LParen => "(",
        LessEqual => "<=",
        Minus => "-",
        MinusAssign => "-=",
        Mod => "%",
        ModAssign => "%=",
        MultAssign => "*=",
        Not => "!",
        NotEqual => "!=",
        Or => "||",
        Plus => "+",
        PlusAssign => "+=",
        Question => "?",
        RAngledBracket => ">",
        RBrace => "}",
        RBracket => "]",
        RParen => ")",
        Semicolon => ";",
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTokenErr;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Punc(Punctuator),
    Keyword(Keyword),
    Ident(String),
    Literal(Literal),
    Unknown(String),
}

impl TokenKind {
    pub fn ident(&self) -> Option<&str> {
        match self {
            Self::Ident(s) => Some(s),
            _ => None,
        }
    }

    pub fn punc(&self) -> Option<&Punctuator> {
        match self {
            Self::Punc(p) => Some(p),
            _ => None,
        }
    }

    pub fn is_assign_op(&self) -> bool {
        matches!(
            self,
            Self::Punc(Punctuator::Assign)
                | Self::Punc(Punctuator::DivAssign)
                | Self::Punc(Punctuator::MinusAssign)
                | Self::Punc(Punctuator::ModAssign)
                | Self::Punc(Punctuator::MultAssign)
                | Self::Punc(Punctuator::PlusAssign)
                | Self::Punc(Punctuator::BitAndAssign)
                | Self::Punc(Punctuator::BitOrAssign)
                | Self::Punc(Punctuator::BitRightShiftAssign)
                | Self::Punc(Punctuator::BitLeftShiftAssign)
                | Self::Punc(Punctuator::BitXorAssign)
        )
    }
}

impl FromStr for TokenKind {
    type Err = ParseTokenErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token = match s {
            _ if let Ok(p) = s.parse::<Punctuator>() => TokenKind::Punc(p),
            _ if let Ok(k) = s.parse::<Keyword>() => TokenKind::Keyword(k),
            _ if let Ok(l) = s.parse::<Literal>() => TokenKind::Literal(l),
            _ if s
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
                && s.chars().all(|c| c.is_alphanumeric() || c == '_') =>
            {
                TokenKind::Ident(s.to_string())
            }
            _ => TokenKind::Unknown(s.to_string()),
        };
        Ok(token)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token = match self {
            Self::Ident(i) => i.clone(),
            Self::Literal(l) => l.to_string(),
            Self::Unknown(u) => u.clone(),
            Self::Keyword(kw) => kw.to_string(),
            Self::Punc(p) => p.to_string(),
        };
        write!(f, "{}", token)
    }
}

impl PartialEq<Keyword> for TokenKind {
    fn eq(&self, other: &Keyword) -> bool {
        matches!(self, Self::Keyword(k) if k == other)
    }
}

impl PartialEq<TokenKind> for Keyword {
    fn eq(&self, other: &TokenKind) -> bool {
        matches!(other, TokenKind::Keyword(k) if k == self)
    }
}

impl PartialEq<Punctuator> for TokenKind {
    fn eq(&self, other: &Punctuator) -> bool {
        matches!(self, Self::Punc(p) if p == other)
    }
}

impl PartialEq<TokenKind> for Punctuator {
    fn eq(&self, other: &TokenKind) -> bool {
        matches!(other, TokenKind::Punc(p) if p == self)
    }
}

impl From<Keyword> for TokenKind {
    fn from(value: Keyword) -> Self {
        TokenKind::Keyword(value)
    }
}

impl From<&Keyword> for TokenKind {
    fn from(value: &Keyword) -> Self {
        TokenKind::Keyword(value.clone())
    }
}

impl From<Punctuator> for TokenKind {
    fn from(value: Punctuator) -> Self {
        TokenKind::Punc(value)
    }
}

impl From<&Punctuator> for TokenKind {
    fn from(value: &Punctuator) -> Self {
        TokenKind::Punc(value.clone())
    }
}
