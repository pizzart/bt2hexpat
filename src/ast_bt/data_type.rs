use std::str::FromStr;

use crate::{
    ast_bt::{
        stmt::{Enum, Expression, Struct},
        token::Ident,
    },
    traits::to_imhex::{ToHexpatErr, ToHexpatStr},
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
    F16,
    F32,
    F64,
    Char,
    DosDate,
    DosTime,
    FileTime,
    TimeT,
    Time64T,
    Guid,
    String,
    Array(Box<DataType>, Option<Box<Expression>>),
    Struct(Struct),
    Enum(Box<Enum>),
    Pointer(Box<DataType>),
    Ident(Ident),
    Args(Box<DataType>, Vec<Expression>),
}

impl DataType {
    pub fn get_size_bytes(&self) -> Option<usize> {
        match self {
            Self::I8 | Self::U8 => Some(1),
            Self::I16 | Self::U16 | Self::F16 => Some(2),
            Self::I32 | Self::U32 | Self::F32 => Some(4),
            Self::I64 | Self::U64 | Self::F64 => Some(4),
            _ => None,
        }
    }
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

    pub fn try_to_imhex_fn_arg(&self) -> Result<String, ToHexpatErr> {
        match self {
            DataType::Array(_, _) => Ok("ref auto".to_string()),
            _ => self.to_hexpat(),
        }
    }

    pub fn try_to_imhex_array(&self) -> Result<String, ToHexpatErr> {
        match self {
            DataType::Array(base_ty, e) => Ok(format!(
                "std::Array<{}, {}>",
                base_ty.try_to_imhex_array()?,
                e.as_ref()
                    .map_or_else(|| Ok(String::new()), |exp| exp.to_hexpat())?
            )),
            _ => self.to_hexpat(),
        }
    }
}

impl FromStr for DataType {
    type Err = ParseDataTypeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "byte" | "int8" => Ok(Self::I8),
            "uchar" | "ubyte" | "uint8" => Ok(Self::U8),
            "short" | "int16" => Ok(Self::I16),
            "ushort" | "uint16" | "word" => Ok(Self::U16),
            "int" | "int32" | "long" => Ok(Self::I32),
            "uint" | "uint32" | "ulong" | "dword" => Ok(Self::U32),
            "int64" | "__int64" | "quad" => Ok(Self::I64),
            "uint64" | "__uint64" | "uquad" | "qword" => Ok(Self::U64),
            "hfloat" => Ok(Self::F16),
            "float" => Ok(Self::F32),
            "double" => Ok(Self::F64),
            "char" => Ok(Self::Char),
            "dosdate" => Ok(Self::DosDate),
            "dostime" => Ok(Self::DosTime),
            "filetime" => Ok(Self::FileTime),
            "time_t" => Ok(Self::TimeT),
            "time_64_t" => Ok(Self::Time64T),
            "guid" => Ok(Self::Guid),
            "string" => Ok(Self::String),
            _ => Err(ParseDataTypeErr),
        }
    }
}

impl ToHexpatStr for DataType {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        match self {
            Self::I8 => Ok("s8".to_owned()),
            Self::U8 => Ok("u8".to_owned()),
            Self::I16 => Ok("s16".to_owned()),
            Self::U16 => Ok("u16".to_owned()),
            Self::I32 => Ok("s32".to_owned()),
            Self::U32 => Ok("u32".to_owned()),
            Self::I64 => Ok("s64".to_owned()),
            Self::U64 => Ok("u64".to_owned()),
            Self::F16 => Ok("type::float16".to_owned()),
            Self::F32 => Ok("float".to_owned()),
            Self::F64 => Ok("double".to_owned()),
            Self::Char => Ok("char".to_owned()),
            Self::DosDate => Ok("type::DOSDate".to_owned()),
            Self::DosTime => Ok("type::DOSTime".to_owned()),
            Self::FileTime => Ok("type::FILETIME".to_owned()),
            Self::TimeT => Ok("type::time_t".to_owned()),
            Self::Time64T => Ok("type::time_64_t".to_owned()),
            Self::Guid => Ok("type::GUID".to_owned()),
            Self::String => Ok("str".to_owned()),
            Self::Struct(s) => s.to_hexpat(),
            Self::Enum(e) => e.to_hexpat(),
            Self::Array(base_ty, _) => base_ty.to_hexpat(),
            Self::Pointer(dt) => Ok(format!("ref {}", dt.to_hexpat()?)),
            Self::Ident(name) => Ok(name.to_hexpat()?),
            Self::Args(name, args) => {
                let args_str = args
                    .iter()
                    .map(|a| a.to_hexpat())
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}<{}>", name.to_hexpat()?, args_str))
            }
        }
    }
}
