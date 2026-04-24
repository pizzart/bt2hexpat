use derive_more::Deref;

use crate::{
    ast::stmt::Expression,
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
                    ImhexAttribute::Color(c) => format!("color({})", c.try_to_imhex()?),
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
