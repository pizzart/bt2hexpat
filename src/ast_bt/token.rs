use std::{fmt, str::FromStr};

use crate::ast_bt::attr::{AttributeType, Color, DisplayFormat};
use crate::ast_bt::{data_type::DataType, literal::Literal};
use crate::str_enum;
use crate::traits::to_imhex::{ToHexpatErr, ToHexpatStr};

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
                datatype.to_hexpat().unwrap(),
                (s if let Ok(s) = s.parse::<DataType>()) => Ok(Self::DataType(s)),
            },
            // Color(color: Color) => {
            //     color.to_string(),
            //     (s if let Ok(s) = s.parse::<Color>()) => Ok(Self::Color(s)),
            // },
            // Attribute(attr: AttributeType) => {
            //     attr.to_string(),
            //     (s if let Ok(s) = s.parse::<AttributeType>()) => Ok(Self::Attribute(s)),
            // },
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

str_enum! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ReservedFunction {
        // not technically a function, but makes it easier to manage this way
        Sizeof => "sizeof",
        RequiresVersion => "RequiresVersion",
        BigEndian => "BigEndian",
        LittleEndian => "LittleEndian",
        FEof => "FEof",
        FTell => "FTell",
        FSeek => "FSeek",
        FileSize => "FileSize",
        Printf => "Printf",
        Warning => "Warning",
        SPrintf => "SPrintf",
        Str => "Str",
        SetBackColor => "SetBackColor",
        ReadByte => "ReadByte",
        ReadDouble => "ReadDouble",
        ReadFloat => "ReadFloat",
        ReadHFloat => "ReadHFloat",
        ReadInt => "ReadInt",
        ReadInt64 => "ReadInt64",
        ReadQuad => "ReadQuad",
        ReadShort => "ReadShort",
        ReadUByte => "ReadUByte",
        ReadUInt => "ReadUInt",
        ReadUInt64 => "ReadUInt64",
        ReadUQuad => "ReadUQuad",
        ReadUShort => "ReadUShort",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Ident {
    Color(Color),
    Attribute(AttributeType),
    DisplayFormat(DisplayFormat),
    Function(ReservedFunction),
    Path(Vec<String>),
    Custom(String),
    #[default]
    Empty,
}

impl Ident {
    pub fn from_path_vec(path: Vec<&'static str>) -> Self {
        Self::Path(path.into_iter().map(|p| p.to_owned()).collect())
    }
}

impl ToHexpatStr for Ident {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        Ok(match self {
            Self::Attribute(a) => a.to_string(),
            Self::DisplayFormat(a) => Literal::from(a).to_hexpat()?,
            Self::Color(c) => Literal::from(c).to_hexpat()?,
            Self::Function(f) => f.to_string(),
            Self::Path(p) => p.iter().enumerate().fold(String::new(), |acc, (i, s)| {
                if i != 0 { acc + "::" + s } else { acc + s }
            }),
            Self::Custom(c) => match c.as_str() {
                "str" => "string0".to_owned(),
                _ => c.to_owned(),
            },
            Self::Empty => "EMPTY".to_owned(),
        })
    }
}

impl FromStr for Ident {
    type Err = ParseTokenErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if let Ok(s) = s.parse::<Color>() => Ok(Self::Color(s)),
            _ if let Ok(s) = s.parse::<AttributeType>() => Ok(Self::Attribute(s)),
            _ if let Ok(s) = s.parse::<DisplayFormat>() => Ok(Self::DisplayFormat(s)),
            _ if let Ok(s) = s.parse::<ReservedFunction>() => Ok(Self::Function(s)),
            _ if s
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
                && s.chars().all(|c| c.is_alphanumeric() || c == '_') =>
            {
                Ok(Self::Custom(s.to_string()))
            }
            _ => Err(ParseTokenErr),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTokenErr;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Punc(Punctuator),
    Keyword(Keyword),
    Ident(Ident),
    Literal(Literal),
    CPPDirective(String),
    Comment(String),
    Unknown(String),
}

impl TokenKind {
    pub fn ident(self) -> Result<Ident, String> {
        match self {
            Self::Ident(s) => Ok(s),
            s => Err(format!("wanted ident, got {:?}", s)),
        }
    }
}

impl FromStr for TokenKind {
    type Err = ParseTokenErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token = match s {
            _ if let Ok(p) = s.parse::<Punctuator>() => TokenKind::Punc(p),
            _ if let Ok(k) = s.parse::<Keyword>() => TokenKind::Keyword(k),
            _ if let Ok(l) = s.parse::<Literal>() => TokenKind::Literal(l),
            _ if let Ok(i) = s.parse::<Ident>() => TokenKind::Ident(i),
            _ => TokenKind::Unknown(s.to_string()),
        };
        Ok(token)
    }
}

impl PartialEq<Keyword> for TokenKind {
    fn eq(&self, other: &Keyword) -> bool {
        matches!(self, Self::Keyword(k) if k == other)
    }
}

impl PartialEq<Punctuator> for TokenKind {
    fn eq(&self, other: &Punctuator) -> bool {
        matches!(self, Self::Punc(p) if p == other)
    }
}

impl From<Keyword> for TokenKind {
    fn from(value: Keyword) -> Self {
        TokenKind::Keyword(value)
    }
}

impl From<&Keyword> for TokenKind {
    fn from(value: &Keyword) -> Self {
        value.clone().into()
    }
}

impl From<Punctuator> for TokenKind {
    fn from(value: Punctuator) -> Self {
        TokenKind::Punc(value)
    }
}

impl From<&Punctuator> for TokenKind {
    fn from(value: &Punctuator) -> Self {
        value.clone().into()
    }
}
