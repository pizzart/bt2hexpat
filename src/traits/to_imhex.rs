use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ToHexpatErr;

impl Error for ToHexpatErr {}

impl fmt::Display for ToHexpatErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed translating binary template to hexpat")
    }
}

pub trait ToHexpatStr {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr>;
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

    fn indent(&self) -> &'static str {
        "    "
    }
}
