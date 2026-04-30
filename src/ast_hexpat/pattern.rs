use crate::{
    ast_bt::stmt::Statement,
    traits::to_imhex::{ToHexpatErr, ToHexpatStr},
};

pub struct HexPattern(pub Vec<Statement>);

impl ToHexpatStr for HexPattern {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        let mut output =
            "#pragma description Converted from 010 Editor Binary Template\n".to_owned();

        output.push_str("\nimport std.array;\n");
        output.push_str("import std.mem;\n");
        output.push_str("import std.io;\n");
        output.push_str("import std.core;\n");
        output.push_str("import std.string;\n");
        output.push_str("import type.float16;\n");
        output.push_str("import type.guid;\n");
        output.push_str("import type.time;\n\n");

        let mut iter = self.0.iter().peekable();
        while let Some(stmt) = iter.next() {
            let newline = if let Some(p) = iter.peek()
                && p.is_oneline()
                && std::mem::discriminant(stmt) == std::mem::discriminant(p)
            {
                "\n"
            } else {
                "\n\n"
            };
            output.push_str(&(stmt.to_hexpat()? + newline));
        }
        Ok(output)
    }
}
