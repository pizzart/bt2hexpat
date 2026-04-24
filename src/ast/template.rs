use crate::{
    ast::stmt::Statement,
    traits::to_imhex::{ToImhex, ToImhexErr},
};

#[derive(Debug, Clone)]
pub struct Template {
    pub statements: Vec<Statement>,
    pub metadata: TemplateMetadata,
}

impl ToImhex for Template {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        let mut output =
            "#pragma description Converted from 010 Editor Binary Template\n".to_owned();

        if let Some(desc) = &self.metadata.description {
            output.push_str(&format!("#pragma description {}\n", desc));
        }
        if let Some(author) = &self.metadata.author {
            output.push_str(&format!("#pragma author {}\n\n", author));
        }
        output.push_str("import std.array;\n");
        output.push_str("import type.float16;\n");
        output.push_str("import type.guid;\n");
        output.push_str("import type.time;\n\n");

        for stmt in self.statements.iter() {
            output.push_str(&(stmt.try_to_imhex()? + "\n\n"));
        }
        Ok(output)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TemplateMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub purpose: Option<String>,
}
