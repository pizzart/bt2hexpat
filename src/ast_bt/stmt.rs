use std::fmt;

use derive_more::Deref;

use crate::{
    ast_bt::{
        attr::{Attributes, Color},
        data_type::DataType,
        literal::Literal,
        token::Punctuator,
    },
    traits::to_imhex::{ToImhex, ToImhexErr},
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
                out.push_str(")");
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

impl ToImhex for Expression {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        match self {
            Self::Literal(lit) => match lit {
                Literal::Binary(b) => Ok(format!("0b{:b}", b)),
                Literal::Decimal(d) => Ok(d.to_string()),
                Literal::Hexadecimal(h) => Ok(format!("0x{:x}", h)),
                Literal::Octal(o) => Ok(format!("0o{:o}", o)),
                Literal::Float(f) => Ok(format!("{}F", f)),
                Literal::Double(d) => Ok(format!("{}D", d)),
                Literal::Char(c) => Ok(format!("'{}'", c)),
                Literal::String(s) => Ok(format!("\"{}\"", s)),
            },
            Self::Identifier(var) => Ok(match var {
                _ if let Ok(c) = var.parse::<Color>() => c.try_to_imhex()?,
                _ => var.to_owned(),
            }),
            Self::UnaryOp(op, expr, pos) => match op {
                Punctuator::Inc => Ok(format!("{} += 1", expr)),
                Punctuator::Dec => Ok(format!("{} -= 1", expr)),
                _ => match pos {
                    UnaryPosition::Prefix => Ok(format!("{}{}", op, expr.try_to_imhex()?)),
                    UnaryPosition::Postfix => Ok(format!("{}{}", expr.try_to_imhex()?, op)),
                },
            },
            Self::BinaryOp(left, op, right) => Ok(format!(
                "{} {} {}",
                match **left {
                    Self::BinaryOp(_, _, _) => format!("({})", left.try_to_imhex()?),
                    _ => left.try_to_imhex()?,
                },
                op,
                match **right {
                    Self::BinaryOp(_, _, _) => format!("({})", right.try_to_imhex()?),
                    _ => right.try_to_imhex()?,
                },
            )),
            Self::Call(name, args) => {
                let args_str = args
                    .iter()
                    .map(|a| a.try_to_imhex())
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}({})", name, args_str))
            }
            Self::Cast(ty, expr) => Ok(format!("{}({})", ty.try_to_imhex()?, expr.try_to_imhex()?)),
            Self::FieldAccess(expr, field) => Ok(format!("{}.{}", expr.try_to_imhex()?, field)),
            Self::ArrayAccess(expr, index) => Ok(format!(
                "{}[{}]",
                expr.try_to_imhex()?,
                index.try_to_imhex()?
            )),
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
    pub fn try_to_imhex_struct(&self) -> Result<String, ToImhexErr> {
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

impl ToImhex for Args {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
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

impl ToImhex for Struct {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
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
                self.body.try_to_imhex()?,
                self.attrs.try_to_imhex_whitespace()?
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub ident: Option<String>,
    pub ty: Option<DataType>,
    pub variants: Vec<(String, Option<Box<Expression>>)>,
    pub attrs: Attributes,
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

        output.push_str(&format!("}}{}", self.attrs.try_to_imhex_whitespace()?));
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct Block(pub Vec<Statement>);

impl ToImhex for Block {
    fn try_to_imhex(&self) -> Result<String, ToImhexErr> {
        let mut output = String::from("{\n");
        for stmt in self.iter() {
            output.push_str(&self.with_indent(&(stmt.try_to_imhex()? + "\n")));
        }
        output.push_str("}");
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

    pub fn is_oneline(&self) -> bool {
        matches!(
            self,
            Self::Assign { .. }
                | Self::Break
                | Self::CPPDirective(_)
                | Self::Continue
                | Self::Expr(_)
                | Self::Return(_)
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
                    output.push_str(&(ty.try_to_imhex()? + " "));
                } else if bits.is_some() && ty.is_int() && ty.is_signed() {
                    output.push_str("signed ");
                }
                output.push_str(ident);

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
                if let Some(p) = pos {
                    output.push_str(&format!(" @ {}", p.try_to_imhex()?));
                }
                if !attrs.is_empty() {
                    output.push_str(&format!("{}", attrs.try_to_imhex_whitespace()?));
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
                args.try_to_imhex()?,
                block.try_to_imhex()?
            )),
            Statement::Assign { left, sign, right } => Ok(format!(
                "{} {} {}",
                left.try_to_imhex()?,
                sign,
                right.try_to_imhex()?
            )),
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
                    match else_stmts.first() {
                        Some(s @ Statement::If { .. }) => output.push_str(&s.try_to_imhex()?),
                        _ => output.push_str(&else_stmts.try_to_imhex()?),
                    }
                }

                Ok(output)
            }
            Statement::While { condition, body } => Ok(format!(
                "while ({}) {}",
                condition.try_to_imhex()?,
                body.try_to_imhex()?
            )),
            Statement::For {
                init,
                test,
                upd,
                body,
            } => Ok(format!(
                "for ({}, {}, {}) {}",
                init.try_to_imhex()?,
                test.try_to_imhex()?,
                upd.try_to_imhex()?,
                body.try_to_imhex()?
            )),
            Statement::Block(block) => block.try_to_imhex(),
            Statement::Return(expr) => expr.as_ref().map_or_else(
                || Ok("return".to_owned()),
                |e| Ok(format!("return {}", e.try_to_imhex()?)),
            ),
            Statement::Break => Ok("break".to_owned()),
            Statement::Continue => Ok("continue".to_owned()),
            Statement::Switch {
                expr,
                cases,
                default,
            } => {
                let mut output = format!("match ({}) {{\n", expr.try_to_imhex()?);
                for (expr, body) in cases.iter() {
                    output.push_str(&self.with_indent(&format!("({}): ", expr.try_to_imhex()?)));

                    // if case_body.0.len() == 1 {
                    //     output.push_str(&case_body.0.get(0).unwrap().try_to_imhex()?);
                    // } else {
                    let mut body = body.clone();
                    if body.0.last() == Some(&Statement::Break) {
                        body.0.remove(body.len() - 1);
                    }
                    output.push_str(&self.with_indent_except_first(&body.try_to_imhex()?));
                    // }
                    output.push_str("\n");
                }
                if let Some(body) = default {
                    output.push_str(&self.with_indent("(_): "));
                    let mut body = body.clone();
                    if body.0.last() == Some(&Statement::Break) {
                        body.0.remove(body.len() - 1);
                    }
                    output.push_str(&self.with_indent_except_first(&body.try_to_imhex()?));
                    output.push_str("\n");
                }
                output.push_str("}");
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
