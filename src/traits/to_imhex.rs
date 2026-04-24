use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ToImhexErr;

impl Error for ToImhexErr {}

impl fmt::Display for ToImhexErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed translating binary template to hexpat")
    }
}

pub trait ToImhex {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr>;
    fn with_indent(&self, s: &str) -> String {
        s.split_inclusive('\n')
            .map(|l| self.indent().to_owned() + l)
            .collect()
    }
    fn with_indent_except_first(&self, s: &str) -> String {
        s.split_inclusive('\n')
            .enumerate()
            .map(|(i, l)| {
                if i == 0 {
                    l.to_owned()
                } else {
                    self.indent().to_owned() + l
                }
            })
            .collect()
    }
    fn with_indent_last(&self, s: &str) -> String {
        s.lines()
            .enumerate()
            .map(|(i, l)| {
                if i == s.lines().count() - 1 {
                    self.with_indent(l) + "\n"
                } else {
                    self.with_indent(l)
                }
            })
            .collect()
    }
    fn indent(&self) -> &'static str {
        "  "
    }
}
