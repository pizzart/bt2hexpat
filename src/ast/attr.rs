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
enum ImhexAttribute {
    Color(Expression),
    Comment(Expression),
    Name(Expression),
    Hidden,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attributes(pub Vec<Attribute>);

impl Attributes {
    pub fn try_to_imhex_whitespace(&self) -> Result<String, ToImhexErr> {
        if self.0.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!(" {}", self.try_to_imhex()?))
        }
    }
}

impl ToImhex for Attributes {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        let mut new_attrs = vec![];
        for attr in self.0.iter() {
            let a = match attr.ty {
                AttributeType::BgColor => Some(ImhexAttribute::Color(attr.value.clone())),
                AttributeType::Comment => Some(ImhexAttribute::Comment(attr.value.clone())),
                AttributeType::Name => Some(ImhexAttribute::Name(attr.value.clone())),
                AttributeType::Hidden => Some(ImhexAttribute::Hidden),
                _ => None,
            };
            if let Some(a) = a {
                new_attrs.push(a);
            }
        }
        if new_attrs.is_empty() {
            Ok(String::new())
        } else {
            let mut output = "[[".to_owned();
            let mut iter = new_attrs.iter().peekable();
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
