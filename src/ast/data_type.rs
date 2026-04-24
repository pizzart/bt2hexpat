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
    DosDate,
    DosTime,
    FileTime,
    TimeT,
    Time64T,
    GUID,
    Array(Box<DataType>, Option<Box<Expression>>),
    Struct(Struct),
    Enum(Box<Enum>),
    Pointer(Box<DataType>),
    Custom(String),
    Unused,
}

impl DataType {
    pub fn is_int(&self) -> bool {
        matches!(
            self,
            Self::I8
                | Self::I16
                | Self::I32
                | Self::I64
                | Self::U8
                | Self::U16
                | Self::U32
                | Self::U64
        )
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64)
    }

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
            Self::DosDate => "DOSDATE",
            Self::DosTime => "DOSTIME",
            Self::FileTime => "FILETIME",
            Self::TimeT => "time_t",
            Self::Time64T => "time_64_t",
            Self::GUID => "GUID",
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
            "dosdate" => Ok(Self::DosDate),
            "dostime" => Ok(Self::DosTime),
            "filetime" => Ok(Self::FileTime),
            "time_t" => Ok(Self::TimeT),
            "time_64_t" => Ok(Self::Time64T),
            "guid" => Ok(Self::GUID),
            _ => Err(ParseDataTypeErr),
        }
    }
}

impl ToImhex for DataType {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        match self {
            Self::I8 => Ok("s8".to_string()),
            Self::U8 => Ok("u8".to_string()),
            Self::I16 => Ok("s16".to_string()),
            Self::U16 => Ok("u16".to_string()),
            Self::I32 => Ok("s32".to_string()),
            Self::U32 => Ok("u32".to_string()),
            Self::I64 => Ok("s64".to_string()),
            Self::U64 => Ok("u64".to_string()),
            Self::Float => Ok("float".to_string()),
            Self::Double => Ok("double".to_string()),
            Self::HFloat => Ok("type::float16".to_string()),
            Self::DosDate => Ok("type::DOSDate".to_owned()),
            Self::DosTime => Ok("type::DOSTime".to_owned()),
            Self::FileTime => Ok("type::FILETIME".to_owned()),
            Self::TimeT => Ok("type::time_t".to_owned()),
            Self::Time64T => Ok("type::time_64_t".to_owned()),
            Self::GUID => Ok("type::GUID".to_owned()),
            Self::Struct(s) => s.try_to_imhex(),
            Self::Enum(e) => e.try_to_imhex(),
            Self::Array(base_ty, _) => base_ty.try_to_imhex(),
            Self::Pointer(dt) => Ok(format!("{} &", dt.try_to_imhex()?)),
            Self::Custom(name) => Ok(name.clone()),
            Self::Unused => Ok("UNUSED".to_string()),
        }
    }
}
