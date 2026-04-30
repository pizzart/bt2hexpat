use std::fmt;

use derive_more::Deref;

use crate::{
    ast_bt::{
        attr::{Attributes, Color},
        data_type::DataType,
        literal::Literal,
        token::Punctuator,
    },
    traits::to_imhex::{ToHexpatErr, ToHexpatStr},
};

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryPosition {
    Prefix,
    Postfix,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    UnaryOp(Punctuator, Box<Expression>, UnaryPosition),
    BinaryOp(Box<Expression>, Punctuator, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Cast(Box<DataType>, Box<Expression>),
    FieldAccess(Box<Expression>, String),
    ArrayAccess(Box<Expression>, Box<Expression>),
    Comment(String),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Literal(s) => s.to_string(),
            Self::Identifier(s) => s.to_string(),
            Self::UnaryOp(s, e, p) => match p {
                UnaryPosition::Prefix => format!("{}{}", s, e),
                UnaryPosition::Postfix => format!("{}{}", e, s),
            },
            Self::BinaryOp(l, s, r) => format!("{}{}{}", l, s, r),
            Self::Call(i, e) => {
                let mut out = format!("{}(", i);
                for (i, expr) in e.iter().enumerate() {
                    out.push_str(&expr.to_string());
                    if i < e.len() - 1 {
                        out.push_str(", ");
                    }
                }
                out.push(')');
                out
            }
            Self::Cast(dt, e) => format!("({}) {}", dt, e),
            Self::FieldAccess(e, s) => format!("{}.{}", e, s),
            Self::ArrayAccess(a, e) => format!("{}[{}]", a, e),
            Self::Comment(s) => s.to_owned(),
        };
        write!(f, "{}", s)
    }
}

impl ToHexpatStr for Expression {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        match self {
            Self::Literal(lit) => lit.to_hexpat(),
            Self::Identifier(var) => Ok(match var {
                _ if let Ok(c) = var.parse::<Color>() => Literal::from(c).to_hexpat()?,
                _ => var.to_owned(),
            }),
            Self::UnaryOp(op, expr, pos) => match op {
                Punctuator::Inc => Ok(format!("{} += 1", expr)),
                Punctuator::Dec => Ok(format!("{} -= 1", expr)),
                _ => match pos {
                    UnaryPosition::Prefix => Ok(format!("{}{}", op, expr.to_hexpat()?)),
                    UnaryPosition::Postfix => Ok(format!("{}{}", expr.to_hexpat()?, op)),
                },
            },
            Self::BinaryOp(left, op, right) => Ok(format!(
                "{} {} {}",
                match **left {
                    Self::BinaryOp(_, _, _) => format!("({})", left.to_hexpat()?),
                    _ => left.to_hexpat()?,
                },
                op,
                match **right {
                    Self::BinaryOp(_, _, _) => format!("({})", right.to_hexpat()?),
                    _ => right.to_hexpat()?,
                },
            )),
            Self::Call(name, args) => {
                let name = name.clone();
                let args_str = args
                    .iter()
                    .map(|a| a.to_hexpat())
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}({})", name, args_str))
            }
            Self::Cast(ty, expr) => Ok(format!("{}({})", ty.to_hexpat()?, expr.to_hexpat()?)),
            Self::FieldAccess(expr, field) => Ok(format!("{}.{}", expr.to_hexpat()?, field)),
            Self::ArrayAccess(expr, index) => {
                Ok(format!("{}[{}]", expr.to_hexpat()?, index.to_hexpat()?))
            }
            Self::Comment(s) => Ok(s.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructType {
    Struct,
    Union,
}

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct Args(pub Vec<(DataType, String)>);

impl Args {
    pub fn try_to_imhex_struct(&self) -> Result<String, ToHexpatErr> {
        let mut output = String::new();
        let mut iter = self.iter().peekable();
        while let Some((_, id)) = iter.next() {
            output.push_str(&format!("auto {}", id));
            if iter.peek().is_some() {
                output.push_str(", ")
            }
        }
        Ok(output)
    }
}

impl ToHexpatStr for Args {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        let mut output = String::new();
        let mut iter = self.iter().peekable();
        while let Some((dt, id)) = iter.next() {
            output.push_str(&format!("{} {}", dt.try_to_imhex_fn_arg()?, id));
            if iter.peek().is_some() {
                output.push_str(", ")
            }
        }
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub ty: StructType,
    pub ident: Option<String>,
    pub args: Args,
    pub body: Block,
    pub attrs: Attributes,
}

impl ToHexpatStr for Struct {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        if self.body.is_empty() {
            if let Some(ref i) = self.ident {
                Ok(format!("using {}", i))
            } else {
                Ok(String::new())
            }
        } else {
            Ok(format!(
                "{} {}{} {}{}",
                if self.body.iter().any(|stmt| matches!(
                    stmt,
                    Statement::VarDef {
                        ident: _,
                        ty: _,
                        value: _,
                        local: _,
                        bits: Some(_),
                        pos: _,
                        attrs: _,
                    }
                )) {
                    "bitfield"
                } else {
                    match self.ty {
                        StructType::Union => "union",
                        StructType::Struct => "struct",
                    }
                },
                self.ident.clone().unwrap_or_else(|| "NONAME".to_string()),
                if !self.args.is_empty() {
                    format!("<{}>", self.args.try_to_imhex_struct()?)
                } else {
                    String::new()
                },
                self.body.to_hexpat()?,
                self.attrs.try_to_imhex_whitespace()?
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub ident: Option<String>,
    pub ty: Option<DataType>,
    pub variants: Vec<(String, Option<Expression>)>,
    pub attrs: Attributes,
}

impl ToHexpatStr for Enum {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        let mut output = format!(
            "enum {} : {} {{\n",
            self.ident.clone().unwrap_or_default(),
            self.ty
                .as_ref()
                .map_or_else(|| Ok("u32".to_string()), |t| t.to_hexpat())?
        );

        for (var_name, value) in &self.variants {
            output.push_str(&self.with_indent(var_name));
            if let Some(v) = value {
                output.push_str(&format!(" = {}", v));
            }
            output.push_str(",\n");
        }

        output.push_str(&format!("}}{}", self.attrs.try_to_imhex_whitespace()?));
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct Block(pub Vec<Statement>);

impl ToHexpatStr for Block {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        let mut output = String::from("{\n");
        for stmt in self.iter() {
            output.push_str(&self.with_indent(&(stmt.to_hexpat()? + "\n")));
        }
        output.push('}');
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDef {
        ident: String,
        ty: DataType,
        value: Option<Expression>,
        local: bool,
        bits: Option<usize>,
        pos: Option<Expression>,
        attrs: Attributes,
    },
    StructDef(Struct),
    EnumDef(Enum),
    TypeDef {
        ident: String,
        ty: DataType,
        attrs: Attributes,
    },
    FnDef {
        ty: DataType,
        ident: String,
        args: Args,
        body: Block,
    },
    Expr(Expression),
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    For {
        init: Expression,
        test: Expression,
        upd: Expression,
        body: Block,
    },
    Switch {
        expr: Expression,
        cases: Vec<(Expression, Block)>,
        default: Option<Block>,
    },
    Break,
    Continue,
    Block(Block),
    Return(Option<Expression>),
    CPPDirective(String),
}

impl Statement {
    pub fn is_with_semicolon(&self) -> bool {
        matches!(
            self,
            Self::EnumDef(_)
                | Self::Expr(_)
                | Self::FnDef { .. }
                | Self::Return(_)
                | Self::StructDef(_)
                | Self::TypeDef { .. }
                | Self::VarDef { .. }
        )
    }

    pub fn is_oneline(&self) -> bool {
        matches!(
            self,
            Self::Break
                | Self::CPPDirective(_)
                | Self::Continue
                | Self::Expr(_)
                | Self::Return(_)
                | Self::TypeDef { .. }
                | Self::VarDef { .. }
        )
    }
}

impl ToHexpatStr for Statement {
    fn to_hexpat(&self) -> Result<String, ToHexpatErr> {
        let s = match self {
            Statement::StructDef(s) => s.to_hexpat(),
            Statement::EnumDef(e) => e.to_hexpat(),
            Statement::TypeDef { ident, ty, attrs } => Ok(format!(
                "using {} = {}{}",
                ident,
                ty.try_to_imhex_array()?,
                attrs.try_to_imhex_whitespace()?
            )),
            Statement::VarDef {
                ident,
                ty,
                value,
                local: _,
                bits,
                pos,
                attrs,
            } => {
                let mut output = String::new();
                if bits.is_none() || matches!(ty, DataType::Enum(_) | DataType::Ident(_)) {
                    output.push_str(&(ty.to_hexpat()? + " "));
                } else if bits.is_some() && ty.is_int() && ty.is_signed() {
                    output.push_str("signed ");
                }
                output.push_str(ident);

                if let DataType::Array(_, size) = ty {
                    if let Some(size_expr) = size {
                        output.push_str(&format!("[{}]", size_expr.to_hexpat()?));
                    } else {
                        output.push_str("[]");
                    }
                }
                if let Some(expr) = value {
                    output.push_str(&format!(" = {}", expr.to_hexpat()?));
                }
                if let Some(b) = bits {
                    output.push_str(&format!(" : {}", b));
                }
                if let Some(p) = pos {
                    output.push_str(&format!(" @ {}", p.to_hexpat()?));
                }
                if !attrs.is_empty() {
                    output.push_str(&attrs.try_to_imhex_whitespace()?);
                }

                Ok(output)
            }
            Statement::FnDef {
                ty: _,
                ident,
                args,
                body: block,
            } => Ok(format!(
                "fn {}({}) {}",
                ident,
                args.to_hexpat()?,
                block.to_hexpat()?
            )),
            Statement::Expr(expr) => expr.to_hexpat(),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let mut output = format!(
                    "if ({}) {}",
                    condition.to_hexpat()?,
                    then_block.to_hexpat()?
                );

                if let Some(else_stmts) = else_block {
                    output.push_str(" else ");
                    match else_stmts.first() {
                        Some(s @ Statement::If { .. }) => output.push_str(&s.to_hexpat()?),
                        _ => output.push_str(&else_stmts.to_hexpat()?),
                    }
                }

                Ok(output)
            }
            Statement::While { condition, body } => Ok(format!(
                "while ({}) {}",
                condition.to_hexpat()?,
                body.to_hexpat()?
            )),
            Statement::For {
                init,
                test,
                upd,
                body,
            } => Ok(format!(
                "for ({}, {}, {}) {}",
                init.to_hexpat()?,
                test.to_hexpat()?,
                upd.to_hexpat()?,
                body.to_hexpat()?
            )),
            Statement::Block(block) => block.to_hexpat(),
            Statement::Return(expr) => expr.as_ref().map_or_else(
                || Ok("return".to_owned()),
                |e| Ok(format!("return {}", e.to_hexpat()?)),
            ),
            Statement::Break => Ok("break".to_owned()),
            Statement::Continue => Ok("continue".to_owned()),
            Statement::Switch {
                expr,
                cases,
                default,
            } => {
                let mut output = format!("match ({}) {{\n", expr.to_hexpat()?);
                for (expr, body) in cases.iter() {
                    output.push_str(&self.with_indent(&format!("({}): ", expr.to_hexpat()?)));

                    // if case_body.0.len() == 1 {
                    //     output.push_str(&case_body.0.get(0).unwrap().try_to_imhex()?);
                    // } else {
                    let mut body = body.clone();
                    if body.0.last() == Some(&Statement::Break) {
                        body.0.remove(body.len() - 1);
                    }
                    output.push_str(&self.with_indent_except_first(&body.to_hexpat()?));
                    // }
                    output.push('\n');
                }
                if let Some(body) = default {
                    output.push_str(&self.with_indent("(_): "));
                    let mut body = body.clone();
                    if body.0.last() == Some(&Statement::Break) {
                        body.0.remove(body.len() - 1);
                    }
                    output.push_str(&self.with_indent_except_first(&body.to_hexpat()?));
                    output.push('\n');
                }
                output.push('}');
                Ok(output)
            }
            Self::CPPDirective(s) => Ok(s.to_owned()),
        };
        if self.is_with_semicolon() {
            s.map(|s| s + ";")
        } else {
            s
        }
    }
}
