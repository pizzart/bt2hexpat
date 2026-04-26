use crate::{
    ast_bt::{data_type::*, stmt::*, template::*},
    traits::to_imhex::{ToImhex, ToImhexErr},
};

pub struct Translator;

impl Translator {
    pub fn new() -> Self {
        Translator
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
        for stmt in def_stmts {
            stmts.insert(after_onelines, stmt);
        }
        let pat = Template {
            metadata: template.metadata.clone(),
            statements: stmts,
        };
        // self.reorder_stmts(&mut template.statements);
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
                    init: init.clone(),
                    test: test.clone(),
                    upd: upd.clone(),
                    body: Block(self.create_statements(&body.0, dest)),
                }),
                Statement::If {
                    condition,
                    then_block,
                    else_block,
                } => stmts.push(Statement::If {
                    condition: condition.clone(),
                    then_block: Block(self.create_statements(&then_block.0, dest)),
                    else_block: else_block
                        .clone()
                        .map(|b| Block(self.create_statements(&b.0, dest))),
                }),
                Statement::EnumDef(e) => {
                    dest.push(Statement::EnumDef(e.clone()));
                }
                Statement::StructDef(s) => {
                    let body = Block(self.create_statements(&s.body.0, dest));
                    dest.push(Statement::StructDef(Struct {
                        ty: s.ty.clone(),
                        ident: s.ident.clone(),
                        args: s.args.clone(),
                        body,
                        attrs: s.attrs.clone(),
                    }));
                }
                Statement::Switch {
                    expr,
                    cases,
                    default,
                } => {
                    let cs = cases
                        .iter()
                        .map(|(e, block)| {
                            (e.clone(), Block(self.create_statements(&block.0, dest)))
                        })
                        .collect();
                    let df = default
                        .clone()
                        .map(|d| Block(self.create_statements(&d.0, dest)));
                    stmts.push(Statement::Switch {
                        expr: expr.clone(),
                        cases: cs,
                        default: df,
                    });
                }
                Statement::TypeDef { ident, ty, attrs } => {
                    if let Some(dt) = self.create_typedef_datatype(ty, dest, ident) {
                        dest.push(Statement::TypeDef {
                            ident: ident.clone(),
                            ty: dt,
                            attrs: attrs.clone(),
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
                    stmts.push(Statement::VarDef {
                        ident: ident.clone(),
                        ty: self.create_datatype(ty, dest),
                        value: value.clone(),
                        local: local.clone(),
                        bits: bits.clone(),
                        pos: pos.clone(),
                        attrs: attrs.clone(),
                    });
                }
                _ => stmts.push(stmt.clone()),
            }
        }
        stmts
    }

    fn create_datatype(&self, src: &DataType, dest: &mut Vec<Statement>) -> DataType {
        match src {
            DataType::Args(dt, e) => {
                DataType::Args(Box::new(self.create_datatype(dt, dest)), e.clone())
            }
            DataType::Array(dt, e) => {
                DataType::Array(Box::new(self.create_datatype(dt, dest)), e.clone())
            }
            DataType::Enum(e) => {
                dest.push(Statement::EnumDef(*e.clone()));
                DataType::Ident(e.ident.clone().unwrap_or_else(|| "NONAME".to_owned()))
            }
            DataType::Struct(s) => {
                dest.push(Statement::StructDef(s.clone()));
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
                    ty: e.ty.clone(),
                    variants: e.variants.clone(),
                    attrs: e.attrs.clone(),
                }));
                None
            }
            DataType::Struct(s) => {
                let st = Statement::StructDef(Struct {
                    ty: s.ty.clone(),
                    ident: Some(ident.to_owned()),
                    args: s.args.clone(),
                    body: Block(self.create_statements(&s.body.0, dest)),
                    attrs: s.attrs.clone(),
                });
                dest.push(st);
                None
            }
            _ => Some(src.clone()),
        }
    }
}
