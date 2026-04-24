use std::fmt;

use crate::{
    ast::{data_type::DataType, literal::Literal, token::Punctuator},
    traits::to_imhex::{ToImhex, ToImhexErr},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    UnaryOp(Punctuator, Box<Expression>),
    BinaryOp(Box<Expression>, Punctuator, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Cast(Box<DataType>, Box<Expression>),
    FieldAccess(Box<Expression>, String),
    ArrayAccess(Box<Expression>, Box<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Literal(s) => s.to_string(),
            Self::Identifier(s) => s.to_string(),
            Self::UnaryOp(s, e) => format!("{}{}", s, e),
            Self::BinaryOp(l, s, r) => format!("{}{}{}", l, s, r),
            Self::Call(i, e) => {
                let mut out = format!("{}(", i);
                for (i, expr) in e.iter().enumerate() {
                    out.push_str(&expr.to_string());
                    if i < e.len() - 1 {
                        out.push_str(", ");
                    }
                }
                out.push_str(")");
                out
            }
            Self::Cast(dt, e) => format!("({}) {}", dt, e),
            Self::FieldAccess(e, s) => format!("{}.{}", e, s),
            Self::ArrayAccess(a, e) => format!("{}[{}]", a, e),
        };
        write!(f, "{}", s)
    }
}

impl ToImhex for Expression {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        match self {
            Expression::Literal(lit) => match lit {
                Literal::Binary(b) => Ok(format!("0b{}", b)),
                Literal::Decimal(d) => Ok(d.to_string()),
                Literal::Hexadecimal(h) => Ok(format!("0x{}", h)),
                Literal::Octal(o) => Ok(format!("0o{}", o)),
                Literal::Float(f) => Ok(format!("{}F", f)),
                Literal::Double(d) => Ok(format!("{}D", d)),
                Literal::Char(c) => Ok(format!("'{}'", c)),
                Literal::String(s) => Ok(format!("\"{}\"", s)),
            },
            Expression::Identifier(var) => Ok(var.clone()),
            Expression::UnaryOp(op, right) => Ok(format!("{}{}", op, right.try_to_imhex()?)),
            Expression::BinaryOp(left, op, right) => Ok(format!(
                "{} {} {}",
                left.try_to_imhex()?,
                op,
                right.try_to_imhex()?
            )),
            Expression::Call(name, args) => {
                let args_str = args
                    .iter()
                    .map(|a| a.try_to_imhex())
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}({})", name, args_str))
            }
            Expression::Cast(ty, expr) => {
                Ok(format!("{}({})", ty.try_to_imhex()?, expr.try_to_imhex()?))
            }
            Expression::FieldAccess(expr, field) => {
                Ok(format!("{}.{}", expr.try_to_imhex()?, field))
            }
            Expression::ArrayAccess(expr, index) => Ok(format!(
                "{}[{}]",
                expr.try_to_imhex()?,
                index.try_to_imhex()?
            )),
        }
    }
}

// #[derive(Debug, Clone, PartialEq)]
// pub enum StructItem {
//     Field(StructField),
//     Statement(Box<Statement>),
// }

// impl ToImhex for StructItem {
//     fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
//         match self {
//             Self::Field(_) => todo!("this enum with just statements"),
//             Self::Statement(s) => s.try_to_imhex(),
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct StructField {
//     pub ident: String,
//     pub ty: DataType,
// }

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub ident: Option<String>,
    pub body: Block,
}

impl ToImhex for Struct {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        Ok(format!(
            "{} {} {}",
            if self.body.0.iter().any(|stmt| matches!(
                stmt,
                Statement::VarDef {
                    ident: _,
                    ty: _,
                    value: _,
                    local: _,
                    bits: Some(_)
                }
            )) {
                "bitfield"
            } else {
                "struct"
            },
            self.ident.clone().unwrap_or_else(|| "NONAME".to_string()),
            self.body.try_to_imhex()?
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub ident: Option<String>,
    pub ty: Option<DataType>,
    pub variants: Vec<(String, Option<Box<Expression>>)>,
}

impl ToImhex for Enum {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        let mut output = format!(
            "enum {} : {} {{\n",
            self.ident.clone().unwrap_or_else(|| String::new()),
            self.ty
                .as_ref()
                .map_or_else(|| Ok("u32".to_string()), |t| t.try_to_imhex())?
        );

        for (var_name, value) in &self.variants {
            output.push_str(&self.with_indent(var_name));
            if let Some(v) = value {
                output.push_str(&format!(" = {}", v));
            }
            output.push_str(",\n");
        }

        output.push_str("}");
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block(pub Vec<Statement>);

impl ToImhex for Block {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        let mut output = String::from("{\n");
        for stmt in self.0.iter() {
            output.push_str(&self.with_indent(&(stmt.try_to_imhex()? + "\n")));
        }
        output.push_str("}");
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Args(pub Vec<(DataType, String)>);

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDef {
        ident: String,
        ty: DataType,
        value: Option<Expression>,
        local: bool,
        bits: Option<usize>,
    },
    Expr(Expression),
    StructDef(Struct),
    TypeDef {
        ident: String,
        ty: DataType,
    },
    EnumDef(Enum),
    FnDef {
        ty: DataType,
        ident: String,
        args: Args,
        block: Block,
    },
    Assign {
        left: Expression,
        sign: String,
        right: Expression,
    },
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    Switch {
        expr: Expression,
        cases: Vec<Block>,
    },
    Break,
    Continue,
    Block(Block),
    Return(Option<Expression>),
}

impl Statement {
    pub fn is_with_semicolon(&self) -> bool {
        matches!(
            self,
            Self::Assign { .. }
                | Self::EnumDef(_)
                | Self::Expr(_)
                | Self::FnDef { .. }
                | Self::Return(_)
                | Self::StructDef(_)
                | Self::TypeDef { .. }
                | Self::VarDef { .. }
        )
    }
}

impl ToImhex for Statement {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        let s = match self {
            Statement::StructDef(s) => s.try_to_imhex(),
            Statement::EnumDef(e) => e.try_to_imhex(),
            Statement::TypeDef { ident, ty } => {
                Ok(format!("using {} = {}", ident, ty.try_to_imhex_braced()?))
            }
            Statement::VarDef {
                ident,
                ty,
                value,
                local: _,
                bits,
            } => {
                let mut output = format!("{} {}", ty.try_to_imhex()?, ident);

                if let DataType::Array(_, size) = ty {
                    if let Some(size_expr) = size {
                        output.push_str(&format!("[{}]", size_expr.try_to_imhex()?));
                    } else {
                        output.push_str("[]");
                    }
                }
                if let Some(expr) = value {
                    output.push_str(&format!(" = {}", expr.try_to_imhex()?));
                }
                if let Some(b) = bits {
                    output.push_str(&format!(" : {}", b));
                }

                Ok(output)
            }
            Statement::FnDef {
                ty: _,
                ident,
                args,
                block,
            } => {
                let mut output = format!("fn {}(", ident);

                for (i, (dt, id)) in args.0.iter().enumerate() {
                    output.push_str(&format!("{} {}", dt.try_to_imhex_fn_arg()?, id));
                    if i < args.0.len() - 1 {
                        output.push_str(", ")
                    }
                }

                output.push_str(") ");
                output.push_str(&block.try_to_imhex()?);

                Ok(output)
            }
            Statement::Assign { left, sign, right } => {
                let l = left.try_to_imhex()?;
                let r = right.try_to_imhex()?;
                Ok(format!("{} {} {}", l, sign, r))
            }
            Statement::Expr(expr) => expr.try_to_imhex(),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let mut output = format!(
                    "if ({}) {}",
                    condition.try_to_imhex()?,
                    then_block.try_to_imhex()?
                );

                if let Some(else_stmts) = else_block {
                    output.push_str(" else ");
                    output.push_str(&else_stmts.try_to_imhex()?);
                }

                Ok(output)
            }
            Statement::While { condition, body } => {
                let mut output = format!("while ({}) ", condition.try_to_imhex()?);
                output.push_str(&body.try_to_imhex()?);
                Ok(output)
            }
            Statement::Block(block) => block.try_to_imhex(),
            Statement::Return(expr) => expr.as_ref().map_or_else(
                || Ok("return".to_owned()),
                |e| Ok(format!("return {}", e.try_to_imhex()?)),
            ),
            Statement::Break => Ok("break".to_owned()),
            Statement::Continue => Ok("continue".to_owned()),
            Statement::Switch { expr, cases } => {
                let mut output = format!("match ({}) {{\n", expr.try_to_imhex()?);
                for (i, case_body) in cases.iter().enumerate() {
                    if i == cases.len() - 1 && !cases.is_empty() {
                        output.push_str(&self.with_indent("(_): "));
                    } else {
                        output.push_str(&self.with_indent(&format!("({}): ", i)));
                    }

                    // if case_body.0.len() == 1 {
                    //     output.push_str(&case_body.0.get(0).unwrap().try_to_imhex()?);
                    // } else {
                    let mut body = case_body.clone();
                    if case_body.0.last() == Some(&Statement::Break) {
                        body.0.remove(case_body.0.len() - 1);
                    }
                    output.push_str(&self.with_indent_except_first(&body.try_to_imhex()?));
                    // }
                    output.push_str("\n");
                }
                output.push_str("}");
                Ok(output)
            }
        };
        if self.is_with_semicolon() {
            s.map(|s| s + ";")
        } else {
            s
        }
    }
}
