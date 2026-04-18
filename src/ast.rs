use crate::str_enum;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    HFloat,
    Float,
    Double,
    Array(Box<DataType>, Option<Box<Expression>>),
    Struct(Struct),
    Enum(Box<Enum>),
    Custom(String),
    Unused,
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
pub struct Struct {
    pub ident: Option<String>,
    pub body: Vec<StructItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub ident: String,
    pub ty: Option<DataType>,
    pub variants: Vec<(String, Option<i64>)>,
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
    StructDef(Struct),
    TypeDef {
        ident: String,
        ty: DataType,
    },
    EnumDef(Enum),
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
    // FunctionCall(String, Vec<Expression>),
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

impl DataType {
    pub fn into_unsigned(self) -> Self {
        match self {
            Self::I8 => Self::U8,
            Self::I16 => Self::U16,
            Self::I32 => Self::U32,
            Self::I64 => Self::U64,
            _ => self,
        }
    }

    pub fn into_signed(self) -> Self {
        match self {
            Self::U8 => Self::I8,
            Self::U16 => Self::I16,
            Self::U32 => Self::I32,
            Self::U64 => Self::I64,
            _ => self,
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::I8 => "int8",
            Self::U8 => "uint8",
            Self::I16 => "int16",
            Self::U16 => "uint16",
            Self::I32 => "int32",
            Self::U32 => "uint32",
            Self::I64 => "int64",
            Self::U64 => "uint64",
            Self::HFloat => "hfloat",
            Self::Float => "float",
            Self::Double => "double",
            Self::Array(dt, size) => &format!(
                "{}{}",
                dt,
                match size {
                    Some(e) => format!("[{}]", e),
                    None => "[]".to_string(),
                }
            ),
            Self::Struct(s) => &format!(
                "struct {} {{\n{}}}",
                s.ident.clone().unwrap_or_else(|| "NONAME".to_string()),
                s.body
                    .iter()
                    .fold(String::new(), |a, _| format!("{}{}", a, "structitem\n"))
            ),
            Self::Enum(e) => &format!(
                "enum <{}> {} {{}}",
                e.ty.as_ref().map(|e| e.to_string()).unwrap_or_default(),
                e.ident
            ),
            Self::Custom(s) => s,
            Self::Unused => "UNUSED",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for DataType {
    type Err = ParseTokenErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "char" | "byte" | "int8" => Ok(Self::I8),
            "uchar" | "ubyte" | "uint8" => Ok(Self::U8),
            "short" | "int16" => Ok(Self::I16),
            "ushort" | "uint16" | "word" => Ok(Self::U16),
            "int" | "int32" | "long" => Ok(Self::I32),
            "uint" | "uint32" | "ulong" | "dword" => Ok(Self::U32),
            "int64" | "__int64" | "quad" => Ok(Self::I64),
            "uint64" | "__uint64" | "uquad" | "qword" => Ok(Self::U64),
            "hfloat" => Ok(Self::Float),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
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
