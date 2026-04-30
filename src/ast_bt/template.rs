use crate::ast_bt::stmt::Statement;

#[derive(Debug, Clone)]
pub struct BinaryTemplate {
    pub statements: Vec<Statement>,
    pub metadata: TemplateMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct TemplateMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub purpose: Option<String>,
}
