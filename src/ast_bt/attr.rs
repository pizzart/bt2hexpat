use derive_more::Deref;

use crate::{
    ast_bt::{literal::Literal, stmt::Expression},
    str_enum,
    traits::to_imhex::{ToHexpatErr, ToHexpatStr},
};

str_enum! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum AttributeType {
        Format => "format",
        FgColor => "fgcolor",
        BgColor => "bgcolor",
        Style => "style",
        Comment => "comment",
        Name => "name",
        Open => "open",
        Hidden => "hidden",
        Read => "read",
        Write => "write",
        Size => "size",
        Edit => "edit",
        Pos => "pos",
        LocalPos => "localpos",
        Optimize => "optimize",
        Disasm => "disasm",
        Warn => "warn",
    }
}

str_enum! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DisplayFormat {
        Hex => "hex",
        Decimal => "decimal",
        Binary => "binary",
        Octal => "octal",
        DecimalHex => "decimalhex",
    }
}

impl From<&DisplayFormat> for Literal {
    fn from(value: &DisplayFormat) -> Self {
        Literal::String(
            match value {
                DisplayFormat::Binary => "type::impl::format_bin",
                DisplayFormat::Decimal => "type::impl::format_dec",
                DisplayFormat::DecimalHex => "type::impl::format_dec",
                DisplayFormat::Hex => "type::impl::format_hex",
                DisplayFormat::Octal => "type::impl::format_oct",
            }
            .to_owned(),
        )
    }
}

str_enum! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Color {
        Black => "cBlack",
        Red => "cRed",
        DarkRed => "cDkRed",
        LightRed => "cLtRed",
        Green => "cGreen",
        DarkGreen => "cDkGreen",
        LightGreen => "cLtGreen",
        Blue => "cBlue",
        DarkBlue => "cDkBlue",
        LightBlue => "cLtBlue",
        Purple => "cPurple",
        DarkPurple => "cDkPurple",
        LightPurple => "cLtPurple",
        Aqua => "cAqua",
        DarkAqua => "cDkAqua",
        LightAqua => "cLtAqua",
        Yellow => "cYellow",
        DarkYellow => "cDkYellow",
        LightYellow => "cLtYellow",
        DarkGray => "cDkGray",
        Gray => "cGray",
        Silver => "cSilver",
        LightGray => "cLtGray",
        White => "cWhite",
        None => "cNone",
    }
}

impl From<&Color> for Literal {
    fn from(value: &Color) -> Self {
        Literal::String(
            match value {
                Color::Black => "000000",
                Color::Red => "ff0000",
                Color::DarkRed => "000080",
                Color::LightRed => "ff8080",
                Color::Green => "00ff00",
                Color::DarkGreen => "008000",
                Color::LightGreen => "80ff80",
                Color::Blue => "0000ff",
                Color::DarkBlue => "000080",
                Color::LightBlue => "8080ff",
                Color::Purple => "ff00ff",
                Color::DarkPurple => "800080",
                Color::LightPurple => "ff80ff",
                Color::Aqua => "00ffff",
                Color::DarkAqua => "008080",
                Color::LightAqua => "80ffff",
                Color::Yellow => "ffff00",
                Color::DarkYellow => "808000",
                Color::LightYellow => "ffff80",
                Color::DarkGray => "404040",
                Color::Gray => "808080",
                Color::Silver => "0c0c0c",
                Color::LightGray => "0e0e0e",
                Color::White => "ffffff",
                Color::None => "000000",
            }
            .to_owned(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub ty: AttributeType,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImhexAttribute {
    Color(Expression),
    Comment(Expression),
    Name(Expression),
    Format(Expression),
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct ImhexAttributes(pub Vec<ImhexAttribute>);

impl ToHexpatStr for ImhexAttributes {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        if self.is_empty() {
            Ok(String::new())
        } else {
            let mut output = "[[".to_owned();
            let mut iter = self.iter().peekable();
            while let Some(attr) = iter.next() {
                let a = match attr {
                    ImhexAttribute::Color(c) => format!("color({})", c.to_hexpat()?),
                    ImhexAttribute::Comment(c) => format!("comment({})", c.to_hexpat()?),
                    ImhexAttribute::Name(n) => format!("name({})", n.to_hexpat()?),
                    ImhexAttribute::Format(n) => format!("format({})", n.to_hexpat()?),
                    ImhexAttribute::Hidden => "hidden".to_owned(),
                };
                output.push_str(&a);
                if iter.peek().is_some() {
                    output.push_str(", ");
                }
            }
            output.push_str("]]");
            Ok(output)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct Attributes(pub Vec<Attribute>);

impl Attributes {
    pub fn contains_type(&self, ty: &AttributeType) -> bool {
        self.iter().any(|attr| &attr.ty == ty)
    }

    pub fn try_to_imhex_whitespace(&self) -> Result<String, ToHexpatErr> {
        let attrs = self.to_imhex_attrs();
        if attrs.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!(" {}", attrs.to_hexpat()?))
        }
    }

    pub fn to_imhex_attrs(&self) -> ImhexAttributes {
        ImhexAttributes(
            self.iter()
                .filter_map(|attr| match attr.ty {
                    AttributeType::BgColor => Some(ImhexAttribute::Color(attr.value.clone())),
                    AttributeType::Comment => Some(ImhexAttribute::Comment(attr.value.clone())),
                    AttributeType::Name => Some(ImhexAttribute::Name(attr.value.clone())),
                    // AttributeType::Format => Some(ImhexAttribute::Format(attr.value.clone())),
                    AttributeType::Hidden => Some(ImhexAttribute::Hidden),
                    _ => None,
                })
                .collect(),
        )
    }
}

impl ToHexpatStr for Attributes {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        self.to_imhex_attrs().to_hexpat()
    }
}
