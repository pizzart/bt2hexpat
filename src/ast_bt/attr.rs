use derive_more::Deref;

use crate::{
    ast_bt::stmt::Expression,
    str_enum,
    traits::to_imhex::{ToImhex, ToImhexErr},
};

str_enum! {
    #[derive(Debug, Clone, PartialEq)]
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
    #[derive(Debug, Clone, PartialEq)]
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

impl ToImhex for Color {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        Ok(match self {
            Self::Black => "000000",
            Self::Red => "ff0000",
            Self::DarkRed => "000080",
            Self::LightRed => "ff8080",
            Self::Green => "00ff00",
            Self::DarkGreen => "008000",
            Self::LightGreen => "80ff80",
            Self::Blue => "0000ff",
            Self::DarkBlue => "000080",
            Self::LightBlue => "8080ff",
            Self::Purple => "ff00ff",
            Self::DarkPurple => "800080",
            Self::LightPurple => "ff80ff",
            Self::Aqua => "00ffff",
            Self::DarkAqua => "008080",
            Self::LightAqua => "80ffff",
            Self::Yellow => "ffff00",
            Self::DarkYellow => "808000",
            Self::LightYellow => "ffff80",
            Self::DarkGray => "404040",
            Self::Gray => "808080",
            Self::Silver => "0c0c0c",
            Self::LightGray => "0e0e0e",
            Self::White => "ffffff",
            Self::None => "000000",
        }
        .to_owned())
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
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct ImhexAttributes(pub Vec<ImhexAttribute>);

impl ToImhex for ImhexAttributes {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        if self.is_empty() {
            Ok(String::new())
        } else {
            let mut output = "[[".to_owned();
            let mut iter = self.iter().peekable();
            while let Some(attr) = iter.next() {
                let a = match attr {
                    ImhexAttribute::Color(c) => format!("color(\"{}\")", c.try_to_imhex()?),
                    ImhexAttribute::Comment(c) => format!("comment({})", c.try_to_imhex()?),
                    ImhexAttribute::Name(n) => format!("name({})", n.try_to_imhex()?),
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
    pub fn try_to_imhex_whitespace(&self) -> Result<String, ToImhexErr> {
        let attrs = self.to_imhex_attrs();
        if attrs.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!(" {}", attrs.try_to_imhex()?))
        }
    }

    pub fn to_imhex_attrs(&self) -> ImhexAttributes {
        ImhexAttributes(
            self.iter()
                .filter_map(|attr| match attr.ty {
                    AttributeType::BgColor => Some(ImhexAttribute::Color(attr.value.clone())),
                    AttributeType::Comment => Some(ImhexAttribute::Comment(attr.value.clone())),
                    AttributeType::Name => Some(ImhexAttribute::Name(attr.value.clone())),
                    AttributeType::Hidden => Some(ImhexAttribute::Hidden),
                    _ => None,
                })
                .collect(),
        )
    }
}

impl ToImhex for Attributes {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        self.to_imhex_attrs().try_to_imhex()
    }
}
