#[derive(Debug, Clone)]
pub enum DataType {
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    Quad,
    UQuad,
    Float,
    Double,
    Custom(String),
    Array(Box<DataType>, Option<Box<Expression>>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(String),
    Identifier(String),
    UnaryOp(String, Box<Expression>),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    FunctionCall(String, Vec<Expression>),
    Cast(Box<DataType>, Box<Expression>),
    FieldAccess(Box<Expression>, String),
    ArrayAccess(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum StructContent {
    Field(StructField),
    Statement(Box<Statement>),
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub ident: String,
    pub ty: DataType,
    pub condition: Option<Expression>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl {
        ident: String,
        ty: DataType,
        value: Option<Expression>,
        local: bool,
    },
    Assign {
        left: Expression,
        right: Expression,
    },
    StructDef {
        ident: String,
        fields: Vec<StructContent>, // Changed from Vec<StructField>
    },
    TypeDef {
        ident: String,
        ty: DataType,
    },
    EnumDef {
        ident: String,
        variants: Vec<(String, Option<i64>)>,
    },
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Switch {
        expr: Expression,
        cases: Vec<Vec<Statement>>,
    },
    FunctionCall(String, Vec<Expression>),
    Return(Option<Expression>),
}

#[derive(Debug, Clone)]
pub struct Template {
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
