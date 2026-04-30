use crate::{
    ast_bt::{
        attr::{Attribute, Attributes, Color},
        data_type::*,
        literal::Literal,
        stmt::*,
        template::*,
        token::Punctuator,
    },
    traits::to_imhex::{ToImhex, ToImhexErr},
};

pub struct Translator {
    current_color: Color,
}

impl Translator {
    pub fn new() -> Self {
        Translator {
            current_color: Color::None,
        }
    }

    pub fn translate(&mut self, template: &Template) -> Result<String, ToImhexErr> {
        let mut def_stmts = vec![];
        let mut stmts = self.create_statements(&template.statements, &mut def_stmts);
        let mut after_onelines = 0;
        for (i, stmt) in stmts.iter().enumerate() {
            if !stmt.is_oneline() {
                after_onelines = i;
                break;
            }
        }
        def_stmts.sort_by(|a, b| {
            if a.is_oneline() && !b.is_oneline() {
                std::cmp::Ordering::Greater
            } else if !a.is_oneline() && b.is_oneline() {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
        for stmt in def_stmts {
            stmts.insert(after_onelines, stmt);
        }
        let pat = Template {
            metadata: template.metadata.clone(),
            statements: stmts,
        };
        pat.try_to_imhex()
    }

    fn create_statements(&self, src: &Vec<Statement>, dest: &mut Vec<Statement>) -> Vec<Statement> {
        let mut stmts = vec![];
        for stmt in src {
            match stmt {
                Statement::Block(b) => {
                    stmts.push(Statement::Block(Block(self.create_statements(&b.0, dest))))
                }
                Statement::FnDef {
                    ty,
                    ident,
                    args,
                    body,
                } => stmts.push(Statement::FnDef {
                    ty: self.create_datatype(ty, dest),
                    ident: ident.clone(),
                    args: Args(
                        args.iter()
                            .map(|(dt, i)| (self.create_datatype(dt, dest), i.clone()))
                            .collect(),
                    ),
                    body: Block(self.create_statements(&body.0, dest)),
                }),
                Statement::For {
                    init,
                    test,
                    upd,
                    body,
                } => stmts.push(Statement::For {
                    init: self.create_expression(init),
                    test: self.create_expression(test),
                    upd: self.create_expression(upd),
                    body: Block(self.create_statements(&body.0, dest)),
                }),
                Statement::While { condition, body } => stmts.push(Statement::While {
                    condition: self.create_expression(condition),
                    body: Block(self.create_statements(&body.0, dest)),
                }),
                Statement::If {
                    condition,
                    then_block,
                    else_block,
                } => stmts.push(Statement::If {
                    condition: self.create_expression(condition),
                    then_block: Block(self.create_statements(&then_block.0, dest)),
                    else_block: else_block
                        .clone()
                        .map(|b| Block(self.create_statements(&b.0, dest))),
                }),
                Statement::EnumDef(e) => {
                    dest.push(Statement::EnumDef(self.create_enum(e)));
                }
                Statement::StructDef(s) => {
                    let st = self.create_struct(s, dest);
                    dest.push(Statement::StructDef(st));
                }
                Statement::Switch {
                    expr,
                    cases,
                    default,
                } => {
                    let cs = cases
                        .iter()
                        .map(|(e, block)| {
                            (
                                self.create_expression(e),
                                Block(self.create_statements(&block.0, dest)),
                            )
                        })
                        .collect();
                    let df = default
                        .clone()
                        .map(|d| Block(self.create_statements(&d.0, dest)));
                    stmts.push(Statement::Switch {
                        expr: self.create_expression(expr),
                        cases: cs,
                        default: df,
                    });
                }
                Statement::TypeDef { ident, ty, attrs } => {
                    if let Some(ty) = self.create_typedef_datatype(ty, dest, ident) {
                        dest.push(Statement::TypeDef {
                            ident: ident.clone(),
                            ty,
                            attrs: self.create_attrs(attrs),
                        });
                    }
                }
                Statement::VarDef {
                    ident,
                    ty,
                    value,
                    local,
                    bits,
                    pos,
                    attrs,
                } => {
                    let v = value.as_ref().map(|e| self.create_expression(e));
                    let p = pos.as_ref().map(|e| self.create_expression(e));
                    stmts.push(Statement::VarDef {
                        ident: ident.clone(),
                        ty: self.create_datatype(ty, dest),
                        value: v,
                        local: *local,
                        bits: *bits,
                        pos: p,
                        attrs: self.create_attrs(attrs),
                    });
                }
                Statement::Expr(e) => stmts.push(Statement::Expr(self.create_expression(e))),
                Statement::Return(e) => stmts.push(Statement::Return(
                    e.as_ref().map(|e| self.create_expression(&e)),
                )),
                _ => stmts.push(stmt.clone()),
            }
        }
        stmts
    }

    fn create_attrs(&self, src: &Attributes) -> Attributes {
        let mut new = vec![];
        for attr in src.iter() {
            new.push(Attribute {
                ty: attr.ty.clone(),
                value: self.create_expression(&attr.value),
            });
        }
        Attributes(new)
    }

    fn create_expression(&self, src: &Expression) -> Expression {
        match src {
            Expression::Call(name, args) => match &**name {
                Expression::Identifier(i) => match i.as_str() {
                    "BigEndian" => Expression::Call(
                        Box::new(Expression::Identifier("std::core::set_endian".to_owned())),
                        vec![Expression::Identifier("std::mem::endian::Big".to_owned())],
                    ),
                    "LittleEndian" => Expression::Call(
                        Box::new(Expression::Identifier("std::core::set_endian".to_owned())),
                        vec![Expression::Identifier(
                            "std::mem::endian::Little".to_owned(),
                        )],
                    ),
                    "FEof" => Expression::Call(
                        Box::new(Expression::Identifier("std::core::eof".to_owned())),
                        vec![],
                    ),
                    "FTell" => Expression::Identifier("$".to_owned()),
                    "FileSize" => Expression::Call(
                        Box::new(Expression::Identifier("std::mem::size".to_owned())),
                        vec![],
                    ),
                    "Printf" => Expression::Call(
                        Box::new(Expression::Identifier("std::print".to_owned())),
                        args.clone(),
                    ),
                    "Warning" => Expression::Call(
                        Box::new(Expression::Identifier("std::warning".to_owned())),
                        args.clone(),
                    ),
                    "SPrintf" => Expression::BinaryOp(
                        Box::new(
                            args.get(0)
                                .cloned()
                                .unwrap_or_else(|| Expression::Identifier("NONE".to_owned())),
                        ),
                        Punctuator::Assign,
                        if args.len() > 2 {
                            Box::new(Expression::Call(
                                Box::new(Expression::Identifier("std::format".to_owned())),
                                args[1..].to_vec(),
                            ))
                        } else if let Some(e) = args.get(1) {
                            if matches!(e, Expression::Literal(Literal::String(_))) {
                                Box::new(e.clone())
                            } else {
                                Box::new(Expression::Call(
                                    Box::new(Expression::Identifier(
                                        "std::string::to_string".to_owned(),
                                    )),
                                    vec![e.clone()],
                                ))
                            }
                        } else {
                            Box::new(Expression::Literal(Literal::String(String::new())))
                        },
                    ),
                    "Str" => Expression::Call(
                        Box::new(Expression::Identifier("std::format".to_owned())),
                        args.clone(),
                    ),
                    _ => src.clone(),
                },
                _ => src.clone(),
            },
            Expression::ArrayAccess(e, i) => Expression::ArrayAccess(
                Box::new(self.create_expression(e)),
                Box::new(self.create_expression(i)),
            ),
            Expression::BinaryOp(l, p, r) => Expression::BinaryOp(
                Box::new(self.create_expression(l)),
                p.clone(),
                Box::new(self.create_expression(r)),
            ),
            Expression::Cast(dt, e) => {
                Expression::Cast(dt.clone(), Box::new(self.create_expression(e)))
            }
            Expression::FieldAccess(e, f) => {
                Expression::FieldAccess(Box::new(self.create_expression(e)), f.clone())
            }
            Expression::UnaryOp(p, e, up) => {
                Expression::UnaryOp(p.clone(), Box::new(self.create_expression(e)), up.clone())
            }
            _ => src.clone(),
        }
    }

    fn create_datatype(&self, src: &DataType, dest: &mut Vec<Statement>) -> DataType {
        match src {
            DataType::Args(dt, e) => DataType::Args(
                Box::new(self.create_datatype(dt, dest)),
                e.iter().map(|e| self.create_expression(e)).collect(),
            ),
            DataType::Array(dt, e) => DataType::Array(
                Box::new(self.create_datatype(dt, dest)),
                e.as_ref().map(|e| Box::new(self.create_expression(&e))),
            ),
            DataType::Enum(e) => {
                dest.push(Statement::EnumDef(self.create_enum(e)));
                DataType::Ident(e.ident.clone().unwrap_or_else(|| "NONAME".to_owned()))
            }
            DataType::Struct(s) => {
                let st = self.create_struct(s, dest);
                dest.push(Statement::StructDef(st));
                DataType::Ident(s.ident.clone().unwrap_or_else(|| "NONAME".to_owned()))
            }
            _ => src.clone(),
        }
    }

    fn create_typedef_datatype(
        &self,
        src: &DataType,
        dest: &mut Vec<Statement>,
        ident: &str,
    ) -> Option<DataType> {
        match src {
            DataType::Args(dt, i) => self
                .create_typedef_datatype(dt, dest, ident)
                .map(|dt| DataType::Args(Box::new(dt), i.clone())),
            DataType::Array(dt, e) => self
                .create_typedef_datatype(dt, dest, ident)
                .map(|dt| DataType::Array(Box::new(dt), e.clone())),
            DataType::Enum(e) => {
                dest.push(Statement::EnumDef(Enum {
                    ident: Some(ident.to_owned()),
                    ..self.create_enum(e)
                }));
                None
            }
            DataType::Struct(s) => {
                let st = Statement::StructDef(Struct {
                    ident: Some(ident.to_owned()),
                    ..self.create_struct(s, dest)
                });
                dest.push(st);
                None
            }
            _ => Some(src.clone()),
        }
    }

    fn create_enum(&self, src: &Enum) -> Enum {
        Enum {
            ident: src.ident.clone(),
            ty: src.ty.clone(),
            variants: src
                .variants
                .iter()
                .map(|(v, expr)| (v.clone(), expr.as_ref().map(|e| self.create_expression(e))))
                .collect(),
            attrs: self.create_attrs(&src.attrs),
        }
    }

    fn create_struct(&self, src: &Struct, dest: &mut Vec<Statement>) -> Struct {
        Struct {
            ty: src.ty.clone(),
            ident: src.ident.clone(),
            args: src.args.clone(),
            body: Block(self.create_statements(&src.body.0, dest)),
            attrs: self.create_attrs(&src.attrs),
        }
    }
}
