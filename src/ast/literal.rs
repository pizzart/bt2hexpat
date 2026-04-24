use std::{fmt, str::FromStr};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseLiteralErr {
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("{0}")]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error("tried to parse invalid literal")]
    InvalidLiteral,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Char(char),
    String(String),
    Decimal(usize),
    Hexadecimal(usize),
    Octal(usize),
    Binary(usize),
    Float(f64),
    Double(f64),
}

#[allow(dead_code)]
impl Literal {
    pub fn char(&self) -> Option<&char> {
        match self {
            Self::Char(c) => Some(c),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn int(&self) -> Option<&usize> {
        match self {
            Self::Decimal(i) | Self::Binary(i) | Self::Hexadecimal(i) | Self::Octal(i) => Some(i),
            _ => None,
        }
    }

    pub fn floating_point(&self) -> Option<&f64> {
        match self {
            Self::Float(f) | Self::Double(f) => Some(f),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum LiteralSuffix {
    Long,
    Unsigned,
    Float,
    Hex,
}

#[derive(Debug, Clone, PartialEq)]
enum LiteralPrefix {
    Binary,
    Hex,
    Octal,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Literal::Char(c) => format!("'{}'", c),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Decimal(i)
            | Literal::Octal(i)
            | Literal::Binary(i)
            | Literal::Hexadecimal(i) => i.to_string(),
            Literal::Float(f) | Literal::Double(f) => f.to_string(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Literal {
    type Err = ParseLiteralErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let literal = match s {
            // looks like 'c' or '\n'
            _ if (s.len() == 3 || (s.len() == 4 && s.chars().nth(1).unwrap() == '\\'))
                && s.starts_with('\'')
                && s.ends_with('\'') =>
            {
                Literal::Char(
                    s.strip_prefix('\'')
                        .unwrap()
                        .strip_suffix('\'')
                        .unwrap()
                        .parse()
                        .unwrap(),
                )
            }
            // looks like "string"
            _ if s.starts_with('"') && s.ends_with('"') => {
                Literal::String(s[1..s.len() - 1].to_owned())
            }
            // is potentially a number
            _ if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.') => {
                let pref = match &s.get(..=1) {
                    Some(s) => match *s {
                        s @ "0b" => Some((s, LiteralPrefix::Binary)),
                        s @ "0x" => Some((s, LiteralPrefix::Hex)),
                        s if s.starts_with('0') && s.len() > 1 => Some(("0", LiteralPrefix::Octal)),
                        _ => None,
                    },
                    _ => None,
                };
                let suffix = match s.chars().last() {
                    Some('L') => Some(LiteralSuffix::Long),
                    Some('u') => Some(LiteralSuffix::Unsigned),
                    Some('f') => Some(LiteralSuffix::Float),
                    Some('h') => Some(LiteralSuffix::Hex),
                    _ => None,
                };
                let mut token = match suffix {
                    Some(_) => s.strip_suffix(s.chars().last().unwrap()).unwrap(),
                    None => s,
                };
                if let Some((s, _)) = pref {
                    token = token.strip_prefix(s).unwrap();
                }
                if token.is_empty() {
                    token = "0";
                }
                let prefix = pref.map(|(_, v)| v);
                match token {
                    // looks like 0b010100
                    _ if prefix == Some(LiteralPrefix::Binary)
                        && token.chars().all(|c| c == '0' || c == '1') =>
                    {
                        Literal::Binary(usize::from_str_radix(token, 2)?)
                    }
                    // looks like 0xf7ee058 or 90e5h
                    _ if (prefix == Some(LiteralPrefix::Hex)
                        || suffix == Some(LiteralSuffix::Hex))
                        && token.chars().all(|c| c.is_ascii_hexdigit()) =>
                    {
                        Literal::Hexadecimal(usize::from_str_radix(token, 16)?)
                    }
                    _ if token.chars().all(|c| c.is_ascii_digit()) => {
                        match prefix {
                            // looks like 0172
                            Some(LiteralPrefix::Octal) => {
                                Literal::Octal(usize::from_str_radix(token, 8)?)
                            }
                            // looks like 192
                            _ => Literal::Decimal(token.parse()?),
                        }
                    }
                    // looks like 90.49 (double) or 90.49f (float)
                    _ if token.len() > 1
                        && token.split_once('.').is_some_and(|(l, r)| {
                            l.chars().all(|c| c.is_ascii_digit())
                                && r.chars().all(|c| c.is_ascii_digit())
                        }) =>
                    {
                        dbg!(token);
                        match suffix {
                            Some(LiteralSuffix::Float) => Literal::Float(token.parse()?),
                            _ => Literal::Double(token.parse()?),
                        }
                    }
                    _ => return Err(ParseLiteralErr::InvalidLiteral),
                }
            }
            _ => return Err(ParseLiteralErr::InvalidLiteral),
        };
        Ok(literal)
    }
}
