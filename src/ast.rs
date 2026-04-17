use crate::str_enum;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    Quad,
    UQuad,
    Float,
    Double,
    Array(Box<DataType>, Option<Box<Expression>>),
    Struct(String, Vec<StructItem>),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(String),
    Identifier(String),
    UnaryOp(String, Box<Expression>),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    Call(String, Vec<Expression>),
    Cast(Box<DataType>, Box<Expression>),
    FieldAccess(Box<Expression>, String),
    ArrayAccess(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructItem {
    Field(StructField),
    Statement(Box<Statement>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub ident: String,
    pub ty: DataType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block(pub Vec<Statement>);

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl {
        ident: String,
        ty: DataType,
        value: Option<Expression>,
        local: bool,
    },
    Assign {
        left: Expression,
        right: Expression,
    },
    Expr(Expression),
    StructDef {
        ident: String,
        body: Vec<StructItem>,
    },
    TypeDef {
        ident: String,
        ty: DataType,
    },
    EnumDef {
        ident: String,
        variants: Vec<(String, Option<i64>)>,
    },
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    Switch {
        expr: Expression,
        cases: Vec<Block>,
    },
    Block(Block),
    FunctionCall(String, Vec<Expression>),
    Return(Option<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Punc(Punctuator),
    Keyword(Keyword),
    Ident(String),
    Int(u64),
    Float(f64),
    Char(char),
    String(String),
    Unknown(String),
}

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

#[derive(Debug, Clone)]
pub struct Template {
    pub statements: Vec<Statement>,
    pub metadata: TemplateMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct TemplateMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub purpose: Option<String>,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Literal(s) => s.to_string(),
            Self::Identifier(s) => s.to_string(),
            Self::UnaryOp(s, e) => format!("{}{}", s, e),
            Self::BinaryOp(l, s, r) => format!("{}{}{}", l, s, r),
            Self::Call(i, e) => {
                let mut out = format!("{}(", i);
                for (i, expr) in e.iter().enumerate() {
                    out.push_str(&expr.to_string());
                    if i < e.len() - 1 {
                        out.push_str(", ");
                    }
                }
                out.push_str(")");
                out
            }
            Self::Cast(dt, e) => format!("({}) {}", dt, e),
            Self::FieldAccess(e, s) => format!("{}.{}", e, s),
            Self::ArrayAccess(a, e) => format!("{}[{}]", a, e),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Char => "char",
            Self::Double => "double",
            Self::Float => "float",
            Self::Int => "int",
            Self::Long => "long",
            Self::Quad => "quad",
            Self::Short => "short",
            Self::UChar => "uchar",
            Self::UInt => "uint",
            Self::ULong => "ulong",
            Self::UQuad => "uquad",
            Self::UShort => "ushort",
            Self::Array(dt, size) => &format!(
                "{}{}",
                dt,
                match size {
                    Some(e) => format!("[{}]", e),
                    None => "[]".to_string(),
                }
            ),
            Self::Struct(s, items) => &format!(
                "struct {} {{\n{}}}",
                s,
                items
                    .iter()
                    .fold(String::new(), |a, _| format!("{}{}", a, "structitem\n"))
            ),
            Self::Custom(s) => s,
        };
        write!(f, "{}", s)
    }
}

impl FromStr for DataType {
    type Err = ParseTokenErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "char" => Ok(Self::Char),
            "double" => Ok(Self::Double),
            "float" => Ok(Self::Float),
            "int" => Ok(Self::Int),
            "long" => Ok(Self::Long),
            "quad" => Ok(Self::Quad),
            "short" => Ok(Self::Short),
            "uchar" => Ok(Self::UChar),
            "uint" => Ok(Self::UInt),
            "ulong" => Ok(Self::ULong),
            "uquad" => Ok(Self::UQuad),
            "ushort" => Ok(Self::UShort),
            _ => Err(ParseTokenErr),
        }
    }
}

impl TokenKind {
    pub fn ident(&self) -> Option<&str> {
        match self {
            Self::Ident(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTokenErr;

impl FromStr for TokenKind {
    type Err = ParseTokenErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token = match s {
            _ if let Ok(p) = s.parse::<Punctuator>() => TokenKind::Punc(p),
            _ if let Ok(k) = s.parse::<Keyword>() => TokenKind::Keyword(k),
            _ if let Ok(i) = s.parse::<u64>() => TokenKind::Int(i),
            _ if let Ok(f) = s.parse::<f64>() => TokenKind::Float(f),
            _ if s.len() == 3 && s.starts_with('\'') && s.ends_with('\'') => {
                TokenKind::Char(s.chars().nth(1).unwrap())
            }
            _ if s.starts_with('"') && s.ends_with('"') => TokenKind::String(s.to_string()),
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
            Self::Char(c) => c.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Int(i) => i.to_string(),
            Self::Ident(i) => i.clone(),
            Self::String(s) => s.clone(),
            Self::Unknown(u) => u.clone(),
            Self::Keyword(kw) => kw.to_string(),
            Self::Punc(p) => p.to_string(),
        };
        write!(f, "{}", token)
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
