use crate::{
    ast_bt::{
        attr::{Attribute, AttributeType, Attributes, Color},
        data_type::*,
        literal::Literal,
        stmt::*,
        template::*,
        token::{Ident, Punctuator, ReservedFunction},
    },
    ast_hexpat::pattern::HexPattern,
    traits::to_imhex::{ToHexpatErr, ToHexpatStr},
};

pub struct Translator {
    current_color: Option<Literal>,
}

impl Translator {
    pub fn new() -> Self {
        Translator {
            current_color: None,
        }
    }

    pub fn translate(&mut self, template: &BinaryTemplate) -> Result<String, ToHexpatErr> {
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
                std::cmp::Ordering::Less
            } else if !a.is_oneline() && b.is_oneline() {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        for stmt in def_stmts.into_iter().rev() {
            stmts.insert(after_onelines, stmt);
        }
        let pat = HexPattern(stmts);
        pat.to_hexpat()
    }

    fn create_statements(
        &mut self,
        src: &Vec<Statement>,
        dest: &mut Vec<Statement>,
    ) -> Vec<Statement> {
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
                    let p = pos.as_ref().map(|e| self.create_expression(e));
                    if let Ident::Custom(i) = ident
                        && i == "padding"
                        && value.is_none() & !local
                        && bits.is_none()
                    {
                        stmts.push(Statement::Expr(Expression::ArrayAccess(
                            Box::new(Expression::Identifier(ident.clone())),
                            Box::new(match ty {
                                DataType::Array(dt, Some(e)) => Expression::BinaryOp(
                                    e.clone(),
                                    Punctuator::Asterisk,
                                    Box::new(Expression::Literal(Literal::Decimal(
                                        dt.get_size_bytes().unwrap_or(1),
                                    ))),
                                ),
                                DataType::Array(dt, None) => Expression::Literal(Literal::Decimal(
                                    dt.get_size_bytes().unwrap_or(1),
                                )),
                                _ => Expression::Literal(Literal::Decimal(
                                    ty.get_size_bytes().unwrap_or(1),
                                )),
                            }),
                        )));
                    } else {
                        let mut ident = ident.clone();
                        let mut count = 0;
                        for s in stmts.iter() {
                            match s {
                                Statement::VarDef { ident: i, .. } if *i == ident => {
                                    count += 1;
                                }
                                _ => (),
                            }
                        }
                        if count > 0
                            && let Ident::Custom(ref mut i) = ident
                        {
                            i.push_str(count.to_string().as_str());
                        }
                        let v = value.as_ref().map(|e| self.create_expression(e));
                        let mut attrs = attrs.clone();
                        if let Some(c) = &self.current_color
                            && !attrs.contains_type(&AttributeType::BgColor)
                        {
                            attrs.0.push(Attribute {
                                ty: AttributeType::BgColor,
                                value: Expression::Literal(c.clone()),
                            });
                        }
                        stmts.push(Statement::VarDef {
                            ident,
                            ty: self.create_datatype(ty, dest),
                            value: v,
                            local: *local,
                            bits: *bits,
                            pos: p,
                            attrs: self.create_attrs(&attrs),
                        });
                    }
                }
                Statement::Expr(e) => stmts.push(Statement::Expr(self.create_expression(e))),
                Statement::Return(e) => stmts.push(Statement::Return(
                    e.as_ref().map(|e| self.create_expression(e)),
                )),
                _ => stmts.push(stmt.clone()),
            }
        }
        stmts
    }

    fn create_attrs(&mut self, src: &Attributes) -> Attributes {
        let mut new = vec![];
        for attr in src.iter() {
            new.push(Attribute {
                ty: attr.ty.clone(),
                value: self.create_expression(&attr.value),
            });
        }
        Attributes(new)
    }

    fn create_expression(&mut self, src: &Expression) -> Expression {
        match src {
            Expression::Call(name, args) => {
                let args = args.iter().map(|a| self.create_expression(a)).collect();
                match &**name {
                    Expression::Identifier(Ident::Function(i)) => match i {
                        ReservedFunction::BigEndian => Expression::Call(
                            Box::new(Expression::Identifier(Ident::from_path_vec(vec![
                                "std",
                                "core",
                                "set_endian",
                            ]))),
                            vec![Expression::Identifier(Ident::from_path_vec(vec![
                                "std", "mem", "Endian", "Big",
                            ]))],
                        ),
                        ReservedFunction::LittleEndian => Expression::Call(
                            Box::new(Expression::Identifier(Ident::from_path_vec(vec![
                                "std",
                                "core",
                                "set_endian",
                            ]))),
                            vec![Expression::Identifier(Ident::from_path_vec(vec![
                                "std", "mem", "Endian", "Little",
                            ]))],
                        ),
                        ReservedFunction::FEof => Expression::Call(
                            Box::new(Expression::Identifier(Ident::from_path_vec(vec![
                                "std", "mem", "eof",
                            ]))),
                            vec![],
                        ),
                        ReservedFunction::FTell => Expression::DollarOp,
                        ReservedFunction::FileSize => Expression::Call(
                            Box::new(Expression::Identifier(Ident::from_path_vec(vec![
                                "std", "mem", "size",
                            ]))),
                            vec![],
                        ),
                        ReservedFunction::Printf => Expression::Call(
                            Box::new(Expression::Identifier(Ident::from_path_vec(vec![
                                "std", "print",
                            ]))),
                            args,
                        ),
                        ReservedFunction::Warning => Expression::Call(
                            Box::new(Expression::Identifier(Ident::from_path_vec(vec![
                                "std", "warning",
                            ]))),
                            args,
                        ),
                        ReservedFunction::SPrintf => Expression::BinaryOp(
                            Box::new(args.first().cloned().unwrap_or_else(|| {
                                Expression::Identifier(Ident::Custom("NONE".to_owned()))
                            })),
                            Punctuator::Assign,
                            if args.len() > 2 {
                                Box::new(Expression::Call(
                                    Box::new(Expression::Identifier(Ident::from_path_vec(vec![
                                        "std", "format",
                                    ]))),
                                    args[1..].to_vec(),
                                ))
                            } else if let Some(e) = args.get(1) {
                                if matches!(e, Expression::Literal(Literal::String(_))) {
                                    Box::new(e.clone())
                                } else {
                                    Box::new(Expression::Call(
                                        Box::new(Expression::Identifier(Ident::from_path_vec(
                                            vec!["std", "string", "to_string"],
                                        ))),
                                        vec![e.clone()],
                                    ))
                                }
                            } else {
                                Box::new(Expression::Literal(Literal::String(String::new())))
                            },
                        ),
                        ReservedFunction::Str => Expression::Call(
                            Box::new(Expression::Identifier(Ident::from_path_vec(vec![
                                "std", "format",
                            ]))),
                            args,
                        ),
                        ReservedFunction::SetBackColor => {
                            self.current_color = args.first().and_then(|e| match e {
                                Expression::Identifier(Ident::Color(c)) => match c {
                                    Color::None => None,
                                    _ => Some(Literal::from(c)),
                                },
                                Expression::Literal(l) => Some(l.clone()),
                                _ => None,
                            });
                            Expression::Comment(format!(
                                "// SetBackColor({:?})",
                                args.first()
                                    .map_or_else(|| "None".to_owned(), |c| c.to_hexpat().unwrap())
                            ))
                        }
                        ReservedFunction::FSeek => Expression::BinaryOp(
                            Box::new(Expression::DollarOp),
                            Punctuator::Assign,
                            Box::new(args.first().unwrap().clone()),
                        ),
                        ReservedFunction::ReadByte
                        | ReservedFunction::ReadDouble
                        | ReservedFunction::ReadFloat
                        | ReservedFunction::ReadHFloat
                        | ReservedFunction::ReadInt
                        | ReservedFunction::ReadInt64
                        | ReservedFunction::ReadQuad
                        | ReservedFunction::ReadShort
                        | ReservedFunction::ReadUByte
                        | ReservedFunction::ReadUInt
                        | ReservedFunction::ReadUInt64
                        | ReservedFunction::ReadUQuad
                        | ReservedFunction::ReadUShort => {
                            let address = args.first().unwrap_or(&Expression::DollarOp);
                            let dt = match i {
                                ReservedFunction::ReadByte => DataType::I8,
                                ReservedFunction::ReadUByte => DataType::U8,
                                ReservedFunction::ReadShort => DataType::I16,
                                ReservedFunction::ReadUShort => DataType::U16,
                                ReservedFunction::ReadInt => DataType::I32,
                                ReservedFunction::ReadUInt => DataType::U32,
                                ReservedFunction::ReadInt64 | ReservedFunction::ReadQuad => {
                                    DataType::I64
                                }
                                ReservedFunction::ReadUInt64 | ReservedFunction::ReadUQuad => {
                                    DataType::U64
                                }
                                ReservedFunction::ReadHFloat => DataType::F16,
                                ReservedFunction::ReadFloat => DataType::F32,
                                ReservedFunction::ReadDouble => DataType::F64,
                                _ => panic!(),
                            };
                            let size = Expression::Literal(Literal::Decimal(
                                dt.get_size_bytes().unwrap_or_default(),
                            ));
                            let func = if dt.is_signed() {
                                Ident::from_path_vec(vec!["std", "mem", "read_signed"])
                            } else {
                                Ident::from_path_vec(vec!["std", "mem", "read_unsigned"])
                            };
                            Expression::Cast(
                                Box::new(dt),
                                Box::new(Expression::Call(
                                    Box::new(Expression::Identifier(func)),
                                    vec![address.clone(), size],
                                )),
                            )
                        }
                        ReservedFunction::RequiresVersion => Expression::Comment(format!(
                            "// RequiresVersion({:?})",
                            args.first()
                                .map_or_else(|| "None".to_owned(), |c| c.to_hexpat().unwrap())
                        )),
                        _ => Expression::Call(name.clone(), args),
                    },
                    _ => Expression::Call(name.clone(), args),
                }
            }
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

    fn create_datatype(&mut self, src: &DataType, dest: &mut Vec<Statement>) -> DataType {
        match src {
            DataType::Args(dt, e) => DataType::Args(
                Box::new(self.create_datatype(dt, dest)),
                e.iter().map(|e| self.create_expression(e)).collect(),
            ),
            DataType::Array(dt, e) => DataType::Array(
                Box::new(self.create_datatype(dt, dest)),
                e.as_ref().map(|e| Box::new(self.create_expression(e))),
            ),
            DataType::Enum(e) => {
                dest.push(Statement::EnumDef(self.create_enum(e)));
                DataType::Ident(e.ident.clone().unwrap_or_default())
            }
            DataType::Struct(s) => {
                let st = self.create_struct(s, dest);
                dest.push(Statement::StructDef(st));
                DataType::Ident(s.ident.clone().unwrap_or_default())
            }
            _ => src.clone(),
        }
    }

    fn create_typedef_datatype(
        &mut self,
        src: &DataType,
        dest: &mut Vec<Statement>,
        ident: &Ident,
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

    fn create_enum(&mut self, src: &Enum) -> Enum {
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

    fn create_struct(&mut self, src: &Struct, dest: &mut Vec<Statement>) -> Struct {
        Struct {
            ty: src.ty.clone(),
            ident: src.ident.clone(),
            args: src.args.clone(),
            body: Block(self.create_statements(&src.body.0, dest)),
            attrs: self.create_attrs(&src.attrs),
        }
    }
}
