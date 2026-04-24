use std::{fmt, str::FromStr};

use crate::{
    ast::stmt::{Enum, Expression, Struct},
    traits::to_imhex::{ToImhex, ToImhexErr},
};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseDataTypeErr;

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
    Pointer(Box<DataType>),
    Custom(String),
    Unused,
}

impl DataType {
    pub fn to_unsigned(&self) -> Self {
        match self {
            Self::I8 => Self::U8,
            Self::I16 => Self::U16,
            Self::I32 => Self::U32,
            Self::I64 => Self::U64,
            _ => self.clone(),
        }
    }

    pub fn to_signed(&self) -> Self {
        match self {
            Self::U8 => Self::I8,
            Self::U16 => Self::I16,
            Self::U32 => Self::I32,
            Self::U64 => Self::I64,
            _ => self.clone(),
        }
    }

    pub fn try_to_imhex_fn_arg(&self) -> Result<String, ToImhexErr> {
        match self {
            DataType::Array(_, _) => Ok("ref auto".to_string()),
            _ => self.try_to_imhex(),
        }
    }

    pub fn try_to_imhex_braced(&self) -> Result<String, ToImhexErr> {
        match self {
            DataType::Array(base_ty, e) => Ok(format!(
                "{}[{}]",
                base_ty.try_to_imhex_braced()?,
                e.as_ref()
                    .map_or_else(|| Ok(String::new()), |exp| exp.try_to_imhex())?
            )),
            _ => self.try_to_imhex(),
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
                s.ident
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| "NONAME"),
                s.body
                    .0
                    .iter()
                    .fold(String::new(), |a, _| format!("{}{}", a, "structitem\n"))
            ),
            Self::Enum(e) => &format!(
                "enum <{}> {} {{}}",
                e.ty.as_ref().map(|e| e.to_string()).unwrap_or_default(),
                e.ident.clone().unwrap_or_else(|| "NONAME".to_string())
            ),
            Self::Pointer(dt) => &format!("&{}", dt.to_string()),
            Self::Custom(s) => s,
            Self::Unused => "UNUSED",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for DataType {
    type Err = ParseDataTypeErr;

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
            "hfloat" => Ok(Self::HFloat),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            _ => Err(ParseDataTypeErr),
        }
    }
}

impl ToImhex for DataType {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        match self {
            DataType::I8 => Ok("s8".to_string()),
            DataType::U8 => Ok("u8".to_string()),
            DataType::I16 => Ok("s16".to_string()),
            DataType::U16 => Ok("u16".to_string()),
            DataType::I32 => Ok("s32".to_string()),
            DataType::U32 => Ok("u32".to_string()),
            DataType::I64 => Ok("s64".to_string()),
            DataType::U64 => Ok("u64".to_string()),
            DataType::HFloat => Ok("HFLOAT".to_string()),
            DataType::Float => Ok("float".to_string()),
            DataType::Double => Ok("double".to_string()),
            DataType::Struct(s) => s.try_to_imhex(),
            DataType::Enum(e) => e.try_to_imhex(),
            DataType::Array(base_ty, _) => base_ty.try_to_imhex(),
            DataType::Pointer(dt) => Ok(format!("{} &", dt.try_to_imhex()?)),
            DataType::Custom(name) => Ok(name.clone()),
            DataType::Unused => Ok("UNUSED".to_string()),
        }
    }
}
